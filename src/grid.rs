use bevy::{
	prelude::{
		App, AssetServer, BuildChildren, Children, Commands, Component, DespawnRecursiveExt,
		Entity, Handle, IVec2, Image, Plugin, Query, Resource, Transform, Vec2, Vec3, With,
	},
	utils::hashbrown::HashMap,
};
use bevy_ecs_tilemap::{
	prelude::{TilemapId, TilemapTexture},
	tiles::{TileBundle, TilePos, TileStorage},
	TilemapBundle,
};

use crate::{physics::Collider, CHUNK_SIZE, TILE_SIZE};

pub struct Grid;

impl Plugin for Grid {
	fn build(&self, app: &mut App) {
		app.insert_resource(Map(HashMap::new()));
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
	pub fn from_size(position: &Vec2, size: &Vec2) -> Region {
		Region {
			top: position.y + size.y,
			left: position.x,
			bottom: position.y,
			right: position.x + size.x,
		}
	}

	pub fn moved(&self, movement: &Vec2) -> Region {
		Region {
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
		let chunk_coord = coord.as_chunk();
		if let Some(map_chunk) = self.0.get(&(chunk_coord.x_i32(), chunk_coord.y_i32())) {
			let tile_coord = coord.as_tile();
			let x_rem = match tile_coord.x_i32().checked_rem(CHUNK_SIZE.x as i32) {
				Some(v) => v.abs(),
				None => 0,
			};
			let y_rem = match tile_coord.y_i32().checked_rem(CHUNK_SIZE.y as i32) {
				Some(v) => v.abs(),
				None => 0,
			};
			if let Some(tile) = map_chunk.tiles.get(&(x_rem, y_rem)) {
				return Some(tile);
			}
		}
		None
	}
}

pub struct MapChunk {
	tilemap_entity: Entity,
	tiles: HashMap<(i32, i32), Tile>,
}

impl MapChunk {
	fn from_tilemap(chunk_entity: Entity) -> MapChunk {
		MapChunk {
			tilemap_entity: chunk_entity,
			tiles: HashMap::new(),
		}
	}
}

pub enum TileType {
	Generic,
}

pub struct Tile {
	tile_type: TileType,
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
}

impl Coordinate {
	pub fn x_i32(&self) -> i32 {
		match self {
			Coordinate::World { x, y: _ } => *x as i32,
			Coordinate::Tile { x, y: _ } => *x,
			Coordinate::Chunk { x, y: _ } => *x,
		}
	}

	pub fn y_i32(&self) -> i32 {
		match self {
			Coordinate::World { x: _, y } => *y as i32,
			Coordinate::Tile { x: _, y } => *y,
			Coordinate::Chunk { x: _, y } => *y,
		}
	}

	pub fn x_f32(&self) -> f32 {
		match self {
			Coordinate::World { x, y: _ } => *x,
			Coordinate::Tile { x, y: _ } => *x as f32,
			Coordinate::Chunk { x, y: _ } => *x as f32,
		}
	}

	pub fn y_f32(&self) -> f32 {
		match self {
			Coordinate::World { x: _, y } => *y,
			Coordinate::Tile { x: _, y } => *y as f32,
			Coordinate::Chunk { x: _, y } => *y as f32,
		}
	}

	pub fn from_vec2(v: Vec2) -> Coordinate {
		Coordinate::World { x: v.x, y: v.y }
	}

	pub fn as_tile(&self) -> Coordinate {
		match self {
			Coordinate::World { x, y } => Coordinate::Tile {
				x: ((x - (TILE_SIZE.x * 0.5)) / TILE_SIZE.x).ceil() as i32,
				y: ((y - (TILE_SIZE.y * 0.5)) / TILE_SIZE.y).ceil() as i32,
			},
			Coordinate::Chunk { x: _, y: _ } => {
				panic!("Tried to convert chunk coordinate to tile coordinate")
			}
			Coordinate::Tile { x: _, y: _ } => *self,
		}
	}

	pub fn as_chunk(&self) -> Coordinate {
		let chunksize_x_f32 = CHUNK_SIZE.x as f32;
		let chunksize_y_f32 = CHUNK_SIZE.y as f32;
		match self {
			Coordinate::Tile { x, y } => Coordinate::Chunk {
				x: (((*x as f32 - 1.0) - (chunksize_x_f32 * 0.5)) / chunksize_x_f32).ceil() as i32,
				y: (((*y as f32 - 1.0) - (chunksize_y_f32 * 0.5)) / chunksize_y_f32).ceil() as i32,
			},
			Coordinate::World { x: _, y: _ } => {
				let tile_coord = &self.as_tile();
				Coordinate::Chunk {
					x: (((tile_coord.x_f32() - 1.0) - (chunksize_x_f32 * 0.5)) / chunksize_x_f32)
						.ceil() as i32,
					y: (((tile_coord.y_f32() - 1.0) - (chunksize_y_f32 * 0.5)) / chunksize_y_f32)
						.ceil() as i32,
				}
			}
			Coordinate::Chunk { x: _, y: _ } => *self,
		}
	}
}

pub fn spawn_chunk(
	commands: &mut Commands,
	asset_server: &AssetServer,
	chunk_pos: IVec2,
	map: &mut Map,
) -> Entity {
	let tilemap_entity = commands.spawn_empty().id();
	let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
	let mut mapchunk_tiles = HashMap::new();
	//let existing_chunk = map.0.get(&(chunk_pos.x, chunk_pos.y));
	for x in 0..CHUNK_SIZE.x {
		for y in 0..CHUNK_SIZE.y {
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
			mapchunk_tiles.insert(
				(x as i32, y as i32),
				Tile {
					tile_type: TileType::Generic,
				},
			);
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
	if let Some(v) = map.0.get(&(chunk_pos.x, chunk_pos.y)) {
		commands.entity(v.tilemap_entity).despawn_recursive();
	}
	map.0.insert(
		(chunk_pos.x, chunk_pos.y),
		MapChunk {
			tilemap_entity,
			tiles: mapchunk_tiles,
		},
	);
	tilemap_entity
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
