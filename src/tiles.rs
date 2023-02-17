use bevy::{
	prelude::{BuildChildren, Commands, Component, Transform, Vec2, Vec3},
	sprite::SpriteBundle,
};

use crate::{
	grid::{Coordinate, MapTile, Region},
	playerphysics::Collider,
	sprites::Sprites,
	CHUNK_SIZE, TILE_SIZE,
};

#[derive(Component)]
pub struct Tile {
	pub tile_type: TileType,
	pub active: bool,
	pub coordinate: Coordinate,
}

#[derive(Copy, Clone)]
pub enum TileType {
	Gravel,
	Moss,
	Dirt,
}

impl TileType {
	pub fn all() -> Vec<TileType> {
		vec![TileType::Gravel, TileType::Moss, TileType::Dirt]
	}

	pub fn get_name(&self) -> String {
		match self {
			TileType::Gravel => "gravel".to_owned(),
			TileType::Moss => "moss".to_owned(),
			TileType::Dirt => "dirt".to_owned(),
		}
	}

	pub fn is_weighted(&self) -> bool {
		match self {
			TileType::Gravel => true,
			_ => false,
		}
	}

	pub fn is_liquid(&self) -> bool {
		match self {
			_ => false,
		}
	}

	pub fn get_granularity(&self) -> u8 {
		match self {
			TileType::Gravel => 1,
			_ => 0,
		}
	}
}

#[derive(Component)]
pub struct WeightedTile {
	pub granularity: u8,
	pub liquid: bool,
}

#[derive(Component, Ord, Eq, PartialEq, PartialOrd)]
pub struct FallingTile(pub i32);

pub fn create_tile(
	commands: &mut Commands,
	coord: Coordinate,
	tile_type: TileType,
	sprites: &Sprites,
) -> MapTile {
	let tilesize_x_f32 = TILE_SIZE.x as f32;
	let tilesize_y_f32 = TILE_SIZE.y as f32;
	let tile_coord = coord.as_tile_coord();
	let chunklocal_coord = coord.as_chunklocal_coord();
	let chunk_coord = coord.as_chunk_coord();

	let texture_handle = sprites.tiles.get(&tile_type.get_name()).unwrap();
	let mut i = tile_coord.x_i32() * tile_coord.y_i32();
	if i == 0 {
		i = tile_coord.x_i32() + tile_coord.y_i32();
	}

	// George Marsaglia's Xorshift
	i ^= i << 13;
	i ^= i >> 17;
	i ^= i << 5;

	i = i / 10;
	i = i.abs() % texture_handle.len() as i32;

	let tile_entity = commands
		.spawn((
			Tile {
				tile_type: tile_type,
				active: false,
				coordinate: tile_coord,
			},
			SpriteBundle {
				texture: texture_handle.get(i as usize).unwrap().clone(),
				transform: Transform {
					translation: Vec3 {
						x: chunklocal_coord.x_f32() * tilesize_x_f32,
						y: chunklocal_coord.y_f32() * tilesize_y_f32,
						z: 0.0,
					},
					..Default::default()
				},
				..Default::default()
			},
			Collider,
			Region::from_size(
				&Vec2::new(
					(chunklocal_coord.x_f32() * tilesize_x_f32)
						+ (chunk_coord.x_f32() * CHUNK_SIZE.0 as f32 * tilesize_x_f32)
						- (tilesize_x_f32 * 0.5),
					(chunklocal_coord.y_f32() * tilesize_y_f32)
						+ (chunk_coord.y_f32() * CHUNK_SIZE.1 as f32 * tilesize_y_f32)
						- (tilesize_y_f32 * 0.5),
				),
				&Vec2::new(tilesize_x_f32, tilesize_y_f32),
			),
		))
		.id();

	let outline = commands.spawn_empty().id();
	commands.entity(tile_entity).add_child(outline);

	if tile_type.is_weighted() {
		commands.entity(tile_entity).insert(WeightedTile {
			granularity: tile_type.get_granularity(),
			liquid: tile_type.is_liquid(),
		});
	}
	MapTile {
		entity: tile_entity,
		outline,
	}
}
