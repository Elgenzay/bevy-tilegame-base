use crate::{
	physics::{Collider, Position, Velocity},
	players::Player,
	CHUNK_SIZE, RENDER_DISTANCE, TILE_SIZE,
};
use bevy::{
	prelude::{
		App, AssetServer, BuildChildren, Children, Commands, Component, DespawnRecursiveExt,
		Entity, EventReader, Handle, IVec2, Image, Plugin, Query, Res, ResMut, Resource, Transform,
		Vec2, Vec3, VisibilityBundle, With,
	},
	sprite::SpriteBundle,
	transform::TransformBundle,
	utils::hashbrown::HashMap,
};

pub struct Grid;

impl Plugin for Grid {
	fn build(&self, app: &mut App) {
		app.insert_resource(Map(HashMap::new()))
			.add_event::<DestroyTileEvent>()
			.add_system(spawn_nearby_chunks)
			.add_system(destroy_tile);
	}
}

#[derive(Component)]
pub struct Chunk;

#[derive(Component)]
pub struct Region {
	pub top: f32,
	pub left: f32,
	pub bottom: f32,
	pub right: f32,
}

impl Region {
	pub fn from_size(position: &Vec2, size: &Vec2) -> Self {
		Self {
			top: position.y + size.y,
			left: position.x,
			bottom: position.y,
			right: position.x + size.x,
		}
	}

	pub fn moved(&self, movement: &Vec2) -> Self {
		Self {
			top: self.top + movement.y,
			bottom: self.bottom + movement.y,
			left: self.left + movement.x,
			right: self.right + movement.x,
		}
	}
}

#[derive(Resource)]
pub struct Map(HashMap<(i32, i32), MapChunk>);

impl Map {
	pub fn get_tile(&self, coord: Coordinate) -> Option<&Tile> {
		let chunk_coord = coord.as_chunk_coord();
		if let Some(map_chunk) = self.0.get(&(chunk_coord.x_i32(), chunk_coord.y_i32())) {
			let local_coord = coord.as_chunklocal_coord();
			if let Some(tile) = map_chunk
				.tiles
				.get(&(local_coord.x_i32(), local_coord.y_i32()))
			{
				return Some(tile);
			}
		}
		None
	}
}

pub struct MapChunk {
	chunk_entity: Entity,
	tiles: HashMap<(i32, i32), Tile>,
}

impl MapChunk {
	fn from_tilemap(chunk_entity: Entity) -> Self {
		Self {
			chunk_entity: chunk_entity,
			tiles: HashMap::new(),
		}
	}
}

pub enum TileType {
	Generic,
}

pub struct Tile {
	tile_type: TileType,
	entity: Entity,
}

impl Tile {
	fn friendly_name(&self) -> String {
		match self.tile_type {
			TileType::Generic => "Generic".to_owned(),
		}
	}
}

#[derive(Clone, Copy)]
pub enum Coordinate {
	World { x: f32, y: f32 },
	Tile { x: i32, y: i32 },
	Chunk { x: i32, y: i32 },
	ChunkLocal { x: u8, y: u8 },
}

impl Coordinate {
	pub fn x_i32(&self) -> i32 {
		match self {
			Self::World { x, y: _ } => *x as i32,
			Self::ChunkLocal { x, y: _ } => *x as i32,
			Self::Tile { x, y: _ } => *x as i32,
			Self::Chunk { x, y: _ } => *x as i32,
		}
	}

	pub fn y_i32(&self) -> i32 {
		match self {
			Self::World { x: _, y } => *y as i32,
			Self::Tile { x: _, y } => *y,
			Self::Chunk { x: _, y } => *y,
			Self::ChunkLocal { x: _, y } => *y as i32,
		}
	}

	pub fn x_f32(&self) -> f32 {
		match self {
			Self::World { x, y: _ } => *x,
			Self::Tile { x, y: _ } => *x as f32,
			Self::Chunk { x, y: _ } => *x as f32,
			Self::ChunkLocal { x, y: _ } => *x as f32,
		}
	}

	pub fn y_f32(&self) -> f32 {
		match self {
			Self::World { x: _, y } => *y,
			Self::Tile { x: _, y } => *y as f32,
			Self::Chunk { x: _, y } => *y as f32,
			Self::ChunkLocal { x: _, y } => *y as f32,
		}
	}

	pub fn from_vec2(v: Vec2) -> Self {
		Self::World { x: v.x, y: v.y }
	}

	pub fn as_tile_coord(&self) -> Self {
		match self {
			Self::World { x, y } => Self::Tile {
				x: ((x - (TILE_SIZE.x as f32 * 0.5)) / TILE_SIZE.x as f32).ceil() as i32,
				y: ((y - (TILE_SIZE.y as f32 * 0.5)) / TILE_SIZE.y as f32).ceil() as i32,
			},
			Self::Chunk { x: _, y: _ } => {
				panic!("Tried to convert chunk coordinate to tile coordinate")
			}
			Self::ChunkLocal { x: _, y: _ } => {
				panic!("Tried to convert chunklocal coordinate to tile coordinate")
			}
			Self::Tile { x: _, y: _ } => *self,
		}
	}

	pub fn as_chunk_coord(&self) -> Coordinate {
		let chunksize_x_f32 = CHUNK_SIZE.x as f32;
		let chunksize_y_f32 = CHUNK_SIZE.y as f32;
		match self {
			Coordinate::Tile { x, y } => Coordinate::Chunk {
				x: (((*x as f32 - 1.0) - (chunksize_x_f32 * 0.5)) / chunksize_x_f32).ceil() as i32,
				y: (((*y as f32 - 1.0) - (chunksize_y_f32 * 0.5)) / chunksize_y_f32).ceil() as i32,
			},
			Coordinate::World { x: _, y: _ } => {
				let tile_coord = self.as_tile_coord();
				Coordinate::Chunk {
					x: (((tile_coord.x_f32() - 1.0) - (chunksize_x_f32 * 0.5)) / chunksize_x_f32)
						.ceil() as i32,
					y: (((tile_coord.y_f32() - 1.0) - (chunksize_y_f32 * 0.5)) / chunksize_y_f32)
						.ceil() as i32,
				}
			}
			Coordinate::Chunk { x: _, y: _ } => *self,
			Coordinate::ChunkLocal { x: _, y: _ } => {
				panic!("Tried to convert chunklocal coordinate to chunk coordinate")
			}
		}
	}

	pub fn as_chunklocal_coord(&self) -> Self {
		let tile_coord = self.as_tile_coord();
		let chunk_size_x_i32 = CHUNK_SIZE.x as i32;
		let chunk_size_y_i32 = CHUNK_SIZE.y as i32;
		let x = match tile_coord.x_i32().checked_rem(chunk_size_x_i32) {
			Some(v) => {
				if v < 0 {
					v + chunk_size_x_i32
				} else {
					v
				}
			}
			None => 0,
		};
		let y = match tile_coord.y_i32().checked_rem(chunk_size_y_i32) {
			Some(v) => {
				if v < 0 {
					v + chunk_size_y_i32
				} else {
					v
				}
			}
			None => 0,
		};
		Self::ChunkLocal {
			x: x as u8,
			y: y as u8,
		}
	}
}

pub fn spawn_chunk(
	commands: &mut Commands,
	asset_server: &AssetServer,
	chunk_pos: IVec2,
	map: &mut Map,
) -> Entity {
	let chunk_entity = commands.spawn_empty().id();
	let mut mapchunk_tiles = HashMap::new();
	let texture_handle: Handle<Image> = asset_server.load("tile.png");
	let tilesize_x_f32 = TILE_SIZE.x as f32;
	let tilesize_y_f32 = TILE_SIZE.y as f32;
	for x in 0..CHUNK_SIZE.x {
		for y in 0..CHUNK_SIZE.y {
			let tile_entity = commands
				.spawn((
					SpriteBundle {
						texture: texture_handle.clone(),
						transform: Transform {
							translation: Vec3 {
								x: x as f32 * tilesize_x_f32,
								y: y as f32 * tilesize_y_f32,
								z: 0.0,
							},
							..Default::default()
						},
						..Default::default()
					},
					Collider,
					Region::from_size(
						&Vec2::new(
							(x as f32 * tilesize_x_f32)
								+ (chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * tilesize_x_f32)
								- (tilesize_x_f32 * 0.5),
							(y as f32 * tilesize_y_f32)
								+ (chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * tilesize_y_f32)
								- (tilesize_y_f32 * 0.5),
						),
						&Vec2::new(tilesize_x_f32, tilesize_y_f32),
					),
				))
				.id();
			commands.entity(chunk_entity).add_child(tile_entity);
			mapchunk_tiles.insert(
				(x as i32, y as i32),
				Tile {
					tile_type: TileType::Generic,
					entity: tile_entity,
				},
			);
		}
	}

	let transform = Transform::from_translation(Vec3::new(
		chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * tilesize_x_f32,
		chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * tilesize_y_f32,
		0.0,
	));

	commands.entity(chunk_entity).insert((
		VisibilityBundle {
			..Default::default()
		},
		TransformBundle {
			local: transform,
			..Default::default()
		},
		Chunk,
		Region::from_size(
			&Vec2::new(
				transform.translation.x - (tilesize_x_f32 * 0.5),
				transform.translation.y - (tilesize_y_f32 * 0.5),
			),
			&Vec2::new(
				CHUNK_SIZE.x as f32 * tilesize_x_f32,
				CHUNK_SIZE.y as f32 * tilesize_y_f32,
			),
		),
	));
	if let Some(v) = map.0.get(&(chunk_pos.x, chunk_pos.y)) {
		commands.entity(v.chunk_entity).despawn_recursive();
	}
	map.0.insert(
		(chunk_pos.x, chunk_pos.y),
		MapChunk {
			chunk_entity,
			tiles: mapchunk_tiles,
		},
	);
	chunk_entity
}

pub fn despawn_chunk(commands: &mut Commands, chunk_pos: IVec2, map: &mut Map) {
	if let Some(v) = map.0.get(&(chunk_pos.x, chunk_pos.y)) {
		commands.entity(v.chunk_entity).despawn_recursive();
	}
	map.0.remove(&(chunk_pos.x, chunk_pos.y));
}

pub fn region_collides(
	region: &Region,
	q_colliders: &Query<&Region, With<Collider>>,
	q_chunks: &Query<(&Region, &Children), With<Chunk>>,
) -> bool {
	for (chunk_region, chunk_children) in q_chunks {
		if !regions_overlap(chunk_region, region) {
			continue;
		}
		for &child in chunk_children.iter() {
			let tile_region = match q_colliders.get(child) {
				Ok(v) => v,
				Err(_) => continue,
			};
			if !regions_overlap(&tile_region, region) {
				continue;
			}
			return true;
		}
	}
	false
}

fn regions_overlap(region_1: &Region, region_2: &Region) -> bool {
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

fn destroy_tile(
	mut ev_destroy: EventReader<DestroyTileEvent>,
	mut map: ResMut<Map>,
	mut commands: Commands,
) {
	for ev in ev_destroy.iter() {
		let chunk_coord = ev.0.as_chunk_coord();
		if let Some(mapchunk) = map.0.get_mut(&(chunk_coord.x_i32(), chunk_coord.y_i32())) {
			let local_coord = ev.0.as_chunklocal_coord();
			if let Some(tile) = mapchunk
				.tiles
				.remove(&(local_coord.x_i32(), local_coord.y_i32()))
			{
				commands.entity(tile.entity).despawn_recursive();
			}
		}
	}
}

fn spawn_nearby_chunks(
	q_player: Query<(&Player, &Position)>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	for (player, position) in q_player.iter() {
		if let Player::Local = player {
			let current_chunk_coord = Coordinate::from_vec2(position.0).as_chunk_coord();
			for x in (current_chunk_coord.x_i32() - RENDER_DISTANCE.x as i32)
				..(current_chunk_coord.x_i32() + RENDER_DISTANCE.x as i32)
			{
				for y in (current_chunk_coord.y_i32() - RENDER_DISTANCE.y as i32)
					..(current_chunk_coord.y_i32() + RENDER_DISTANCE.y as i32)
				{
					if map.0.contains_key(&(x, y)) {
						continue;
					}
					spawn_chunk(&mut commands, &asset_server, IVec2::new(x, y), &mut map);
				}
			}
		}
	}
}

fn despawn_distant_chunks(
	q_player: Query<(&Player, &Position)>,
	mut map: ResMut<Map>,
	mut commands: Commands,
) {
	for (player, position) in q_player.iter() {
		if let Player::Local = player {
			//
		}
	}
}

pub struct DestroyTileEvent(pub Coordinate);
