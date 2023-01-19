use bevy::{input::Input, math::Vec3, render::camera::Camera};
use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_ecs_tilemap::{
	prelude::{TileStorage, TilemapId, TilemapTexture, TilemapTileSize},
	tiles::{TileBundle, TilePos},
	TilemapBundle, TilemapPlugin,
};

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct Player {
	on_ground: bool,
}

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Chunk;

#[derive(Component)]
struct Region {
	top: f32,
	left: f32,
	bottom: f32,
	right: f32,
}

#[derive(Component, Deref, DerefMut)]
struct Gravity(f32);

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

impl Region {
	fn from_size(position: &Vec2, size: &Vec2) -> Region {
		Region {
			top: position.y + size.y,
			left: position.x,
			bottom: position.y,
			right: position.x + size.x,
		}
	}

	fn moved(&self, movement: &Vec2) -> Region {
		Region {
			top: self.top + movement.y,
			bottom: self.bottom + movement.y,
			left: self.left + movement.x,
			right: self.right + movement.x,
		}
	}
}

const WINDOW_DEFAULT_WIDTH: f32 = 1280.0;
const WINDOW_DEFAULT_HEIGHT: f32 = 720.0;

const CHUNK_SIZE: UVec2 = UVec2 { x: 4, y: 4 };
const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };

const PLAYER_SIZE: UVec2 = UVec2 { x: 24, y: 24 };
const PLAYER_ACCEL: f32 = 1000.0;
const PLAYER_SPEED: f32 = 100.0;
const PLAYER_JUMP_FORCE: f32 = 200.0;
const GROUND_FRICTION: f32 = 10.0;
const AIR_FRICTION: f32 = 10.0;
const AIR_CONTROL: f32 = 0.2;
const GRAVITY_SCALE: f32 = 400.0;
const TERMINAL_VELOCITY: f32 = 500.0;

fn main() {
	App::new()
		.add_plugins(
			DefaultPlugins
				.set(WindowPlugin {
					window: WindowDescriptor {
						width: WINDOW_DEFAULT_WIDTH,
						height: WINDOW_DEFAULT_HEIGHT,
						resizable: true,
						title: String::from("framework"),
						..Default::default()
					},
					..default()
				})
				.set(ImagePlugin::default_nearest()),
		)
		.add_plugin(TilemapPlugin)
		.add_startup_system(startup)
		.add_system(mouse_events_system)
		.add_system(apply_velocity)
		.add_system(apply_gravity)
		.add_system(move_player)
		.run();
}

fn startup(mut commands: Commands, mut windows: ResMut<Windows>, asset_server: Res<AssetServer>) {
	commands.spawn(Camera2dBundle::default());

	commands.spawn((
		SpriteBundle {
			transform: Transform::from_xyz(0.0, 0.0, -100.0),
			texture: asset_server.load("cursor.png"),
			sprite: Sprite {
				..Default::default()
			},
			..default()
		},
		Cursor,
	));
	windows
		.get_primary_mut()
		.unwrap()
		.set_cursor_visibility(false);
	commands.spawn((
		SpriteBundle {
			transform: Transform::from_translation(Vec3::ZERO),
			texture: asset_server.load("player.png"),
			..Default::default()
		},
		Velocity(Vec2::ZERO),
		Gravity(GRAVITY_SCALE),
		Player { on_ground: false },
	));

	spawn_chunk(&mut commands, &asset_server, IVec2::new(0, -2));
}

fn mouse_events_system(
	wnds: Res<Windows>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut query: Query<&mut Transform, With<Cursor>>,
) {
	let (camera, camera_transform) = q_camera.single();
	let wnd = if let RenderTarget::Window(id) = camera.target {
		wnds.get(id).unwrap()
	} else {
		wnds.get_primary().unwrap()
	};
	if let Some(screen_pos) = wnd.cursor_position() {
		let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
		let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
		let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
		let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
		let world_pos: Vec2 = world_pos.truncate();
		let cursorlocation = Vec3::new(world_pos.x.floor(), world_pos.y.floor(), 0.0);
		if query.single_mut().translation != cursorlocation {
			query.single_mut().translation = cursorlocation;
			/*
			eprintln!(
				"({},{})",
				cursorlocation.x.to_string(),
				cursorlocation.y.to_string()
			);
			*/
		}
	}
}

fn apply_velocity(
	time: Res<Time>,
	q_colliders: Query<&Region, With<Collider>>,
	q_chunks: Query<(&Region, &Children), With<Chunk>>,
	mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
	for (mut player_transform, mut player_velocity) in &mut player_query {
		if player_velocity.x == 0.0 && player_velocity.y == 0.0 {
			continue;
		}
		let delta_y = player_velocity.y * time.delta_seconds();
		let new_pos = Vec3::new(
			player_transform.translation.x + (player_velocity.x * time.delta_seconds()),
			player_transform.translation.y + delta_y,
			0.0,
		);
		let new_player_region = Region::from_size(
			&Vec2::new(
				new_pos.x - (PLAYER_SIZE.x as f32 * 0.5),
				new_pos.y - (PLAYER_SIZE.y as f32 * 0.5),
			),
			&PLAYER_SIZE.as_vec2(),
		);
		if !region_collides(&new_player_region, &q_colliders, &q_chunks) {
			player_transform.translation = new_pos;
		} else if player_velocity.x != 0.0 {
			player_velocity.x = 0.0;
			let new_player_region = Region::from_size(
				&Vec2::new(
					player_transform.translation.x - (PLAYER_SIZE.x as f32 * 0.5),
					player_transform.translation.y - (PLAYER_SIZE.y as f32 * 0.5),
				),
				&PLAYER_SIZE.as_vec2(),
			)
			.moved(&Vec2::new(0.0, delta_y));
			if !region_collides(&new_player_region, &q_colliders, &q_chunks) {
				player_transform.translation.y += delta_y;
			} else {
				player_velocity.y = 0.0;
			}
		} else {
			player_velocity.y = 0.0;
		}
	}
}

fn apply_gravity(
	mut query: Query<(&Gravity, &mut Velocity, &Transform, &mut Player)>,
	time: Res<Time>,
	q_colliders: Query<&Region, With<Collider>>,
	q_chunks: Query<(&Region, &Children), With<Chunk>>,
) {
	for (gravity, mut velocity, transform, mut player) in &mut query {
		let floor_check = Region::from_size(
			&Vec2::new(
				transform.translation.x - (PLAYER_SIZE.x as f32 * 0.5),
				transform.translation.y - (PLAYER_SIZE.y as f32 * 0.5),
			),
			&PLAYER_SIZE.as_vec2(),
		)
		.moved(&Vec2::new(0.0, -1.0));
		if region_collides(&floor_check, &q_colliders, &q_chunks) {
			player.on_ground = true;
			if velocity.y < 0.0 {
				velocity.y = 0.0;
			}
			continue;
		}
		player.on_ground = false;
		if velocity.y < -TERMINAL_VELOCITY {
			continue;
		}
		velocity.y = velocity.y - (gravity.0 * time.delta_seconds());
	}
}

fn overlaps(region_1: &Region, region_2: &Region) -> bool {
	if region_1.right < region_2.left {
		return false;
	}
	if region_1.left > region_2.right {
		return false;
	}
	if region_1.top < region_2.bottom {
		return false;
	}
	if region_1.bottom > region_2.top {
		return false;
	}
	true
}

fn region_collides(
	region: &Region,
	q_colliders: &Query<&Region, With<Collider>>,
	q_chunks: &Query<(&Region, &Children), With<Chunk>>,
) -> bool {
	for (chunk_region, chunk_children) in q_chunks {
		if !overlaps(chunk_region, region) {
			continue;
		}
		for &child in chunk_children.iter() {
			let tile_region = match q_colliders.get(child) {
				Ok(v) => v,
				Err(_) => continue,
			};
			if !overlaps(&tile_region, region) {
				continue;
			}
			return true;
		}
	}
	false
}

fn move_player(
	keyboard_input: Res<Input<KeyCode>>,
	mut query: Query<(&Player, &mut Velocity)>,
	time: Res<Time>,
) {
	let mut direction = 0.0;
	if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
		direction -= 1.0;
	}
	if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
		direction += 1.0;
	}
	let (player, mut velocity) = query.single_mut();
	let mut jumping = false;
	if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
		jumping = true;
	};
	if player.on_ground {
		if jumping {
			velocity.y = PLAYER_JUMP_FORCE;
		}
		if direction == 0.0 {
			velocity.x = 0.0;
			return;
		}
		if velocity.x > PLAYER_SPEED {
			velocity.x -= GROUND_FRICTION * time.delta_seconds();
			return;
		}
		if velocity.x < -PLAYER_SPEED {
			velocity.x += GROUND_FRICTION * time.delta_seconds();
			return;
		}
		velocity.x += direction * PLAYER_ACCEL * time.delta_seconds();
		return;
	}
	if velocity.x > 0.0 {
		velocity.x -= AIR_FRICTION * time.delta_seconds();
	} else if velocity.x < 0.0 {
		velocity.x -= AIR_FRICTION * time.delta_seconds();
	}
	if direction != 0.0 {
		velocity.x += direction * PLAYER_ACCEL * AIR_CONTROL * time.delta_seconds();
	}
}

fn spawn_chunk(commands: &mut Commands, asset_server: &AssetServer, chunk_pos: IVec2) -> Entity {
	let tilemap_entity = commands.spawn_empty().id();
	let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
	for x in 0..CHUNK_SIZE.x {
		for y in 0..CHUNK_SIZE.y {
			//////////////////////
			if x == 1 || x == 2 {
				if y != 0 {
					continue;
				}
			}
			//////////////////////
			let tile_pos = TilePos { x, y };
			let tile_entity = commands
				.spawn((
					TileBundle {
						position: tile_pos,
						tilemap_id: TilemapId(tilemap_entity),
						..Default::default()
					},
					Collider,
					Region::from_size(
						&Vec2::new(
							(x as f32 * TILE_SIZE.x)
								+ (chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x as f32)
								- (TILE_SIZE.x * 0.5),
							(y as f32 * TILE_SIZE.y)
								+ (chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y as f32)
								- (TILE_SIZE.y * 0.5),
						),
						&Vec2::new(TILE_SIZE.x, TILE_SIZE.y),
					),
				))
				.id();
			commands.entity(tilemap_entity).add_child(tile_entity);
			tile_storage.set(&tile_pos, tile_entity);
		}
	}

	let transform = Transform::from_translation(Vec3::new(
		chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x,
		chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y,
		0.0,
	));
	let texture_handle: Handle<Image> = asset_server.load("tiles.png");
	commands.entity(tilemap_entity).insert((
		TilemapBundle {
			grid_size: TILE_SIZE.into(),
			size: CHUNK_SIZE.into(),
			storage: tile_storage,
			texture: TilemapTexture::Single(texture_handle),
			tile_size: TILE_SIZE,
			transform,
			..Default::default()
		},
		Chunk,
		Region::from_size(
			&Vec2::new(
				transform.translation.x - (TILE_SIZE.x * 0.5),
				transform.translation.y - (TILE_SIZE.y * 0.5),
			),
			&Vec2::new(
				CHUNK_SIZE.x as f32 * TILE_SIZE.x,
				CHUNK_SIZE.y as f32 * TILE_SIZE.y,
			),
		),
	));
	tilemap_entity
}
