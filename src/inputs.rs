use crate::{
	grid::{Coordinate, DestroyTileEvent, Map},
	players::{Jumping, MoveDirection, Player},
	settings::Settings,
	sprites::Sprites,
	tilephysics::UpdateTileEvent,
	tiles::{create_tile, TileType},
	Cursor,
};
use bevy::{
	prelude::{
		App, Camera, Commands, EventWriter, GlobalTransform, Input, KeyCode, MouseButton, Plugin,
		Query, Res, ResMut, Transform, Vec2, Vec3, With,
	},
	render::camera::RenderTarget,
	window::Windows,
};

pub struct Inputs;

impl Plugin for Inputs {
	fn build(&self, app: &mut App) {
		app.add_system(mouse_events_system)
			.add_system(keyboard_events_system);
	}
}

pub struct KeyBinds {
	pub move_left: KeyBind,
	pub move_right: KeyBind,
	pub jump: KeyBind,
}

impl Default for KeyBinds {
	fn default() -> Self {
		Self {
			move_left: KeyBind {
				primary: Some(KeyCode::A),
				secondary: Some(KeyCode::Left),
			},
			move_right: KeyBind {
				primary: Some(KeyCode::D),
				secondary: Some(KeyCode::Right),
			},
			jump: KeyBind {
				primary: Some(KeyCode::W),
				secondary: Some(KeyCode::Space),
			},
		}
	}
}

pub struct KeyBind {
	pub primary: Option<KeyCode>,
	pub secondary: Option<KeyCode>,
}

impl KeyBind {
	pub fn is_pressed(&self, input: &Input<KeyCode>) -> bool {
		if let Some(v) = self.primary {
			if input.pressed(v) {
				return true;
			}
		}
		if let Some(v) = self.secondary {
			if input.pressed(v) {
				return true;
			}
		}
		false
	}

	pub fn just_pressed(&self, input: &Input<KeyCode>) -> bool {
		if let Some(v) = self.primary {
			if input.just_pressed(v) {
				return true;
			}
		}
		if let Some(v) = self.secondary {
			if input.just_pressed(v) {
				return true;
			}
		}
		false
	}
}

fn keyboard_events_system(
	input: Res<Input<KeyCode>>,
	settings: Res<Settings>,
	mut q_player: Query<(&Player, &mut MoveDirection, &mut Jumping)>,
) {
	for (player, mut move_direction, mut jumping) in &mut q_player {
		if let Player::Local = player {
			let mut dir = MoveDirection::None;
			if settings.keybinds.move_left.is_pressed(&input) {
				dir = MoveDirection::Left;
			}
			if settings.keybinds.move_right.is_pressed(&input) {
				if let MoveDirection::Left = dir {
					dir = MoveDirection::None;
				} else {
					dir = MoveDirection::Right;
				}
			}
			*move_direction = dir;
			if settings.hold_to_keep_jumping {
				jumping.0 = settings.keybinds.jump.is_pressed(&input);
			} else {
				jumping.0 = settings.keybinds.jump.just_pressed(&input);
			}
			return;
		}
	}
}

fn mouse_events_system(
	wnds: Res<Windows>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut q_cursor: Query<&mut Transform, With<Cursor>>,
	input: Res<Input<MouseButton>>,
	mut ev_destroytile: EventWriter<DestroyTileEvent>,
	mut ev_updatetile: EventWriter<UpdateTileEvent>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	sprites: Res<Sprites>,
) {
	let (camera, camera_transform) = match q_camera.get_single() {
		Ok(v) => v,
		Err(_) => return,
	};
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
		let cursorlocation = Vec3::new(world_pos.x.floor(), world_pos.y.floor(), 1.0);
		let world_coord = Coordinate::world_coord_from_vec2(world_pos);
		let mut cursor = match q_cursor.get_single_mut() {
			Ok(v) => v,
			Err(_) => return,
		};
		if cursor.translation != cursorlocation {
			cursor.translation = cursorlocation;
			//let tile = map.get_tile(world_coord);
		}
		if input.pressed(MouseButton::Left) {
			let tile_coord = world_coord.as_tile_coord();
			let bottom_left = tile_coord.moved(&Vec2::NEG_ONE);
			let top_right = tile_coord.moved(&Vec2::ONE);
			for x in bottom_left.x_i32()..=top_right.x_i32() {
				for y in bottom_left.y_i32()..=top_right.y_i32() {
					ev_destroytile.send(DestroyTileEvent(Coordinate::Tile { x, y }));
				}
			}
		}

		let tile_coord = world_coord.as_tile_coord();
		if input.just_pressed(MouseButton::Right) {
			let e = create_tile(&mut commands, tile_coord, TileType::Dirt, &sprites);
			let _ = map.set_tile(&mut commands, tile_coord, Some(e));
		}
		if input.pressed(MouseButton::Right) {
			let tile_coord = world_coord.as_tile_coord();
			let bottom_left = tile_coord.moved(&Vec2::NEG_ONE);
			let top_right = tile_coord.moved(&Vec2::ONE);
			for x in bottom_left.x_i32()..=top_right.x_i32() {
				for y in bottom_left.y_i32()..=top_right.y_i32() {
					if let Ok(opt) = map.get_tile(Coordinate::Tile { x, y }) {
						if let Some(t) = opt {
							ev_updatetile.send(UpdateTileEvent(*t));
						}
					} else {
						//unloaded chunk
					}
				}
			}
		}
	}
}
