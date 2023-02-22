use bevy::{
	prelude::{Commands, Component, EventWriter, Transform, Vec2, Vec3},
	sprite::SpriteBundle,
};

use crate::{
	grid::{Coordinate, Map, MapTile, Region},
	playerphysics::Collider,
	sprites::Sprites,
	tilephysics::UpdateTileEvent,
	tiletypes::TileType,
	CHUNK_SIZE, TILE_SIZE,
};

#[derive(Component)]
pub struct Tile {
	pub tile_type: TileType,
	pub coordinate: Coordinate,
}

#[derive(Component)]
pub struct WeightedTile {
	pub granularity: u8,
	pub liquid: bool,
}

#[derive(Component, Ord, Eq, PartialEq, PartialOrd)]
pub struct FallingTile(pub i32);

pub fn set_tile(
	commands: &mut Commands,
	coord: Coordinate,
	tile_type: TileType,
	sprites: &Sprites,
	map: &mut Map,
	update_tile_event: &mut EventWriter<UpdateTileEvent>,
) -> Result<MapTile, ()> {
	let tilesize_x_f32 = TILE_SIZE.x as f32;
	let tilesize_y_f32 = TILE_SIZE.y as f32;
	let tile_coord = coord.as_tile_coord();
	let chunklocal_coord = coord.as_chunklocal_coord();
	let chunk_coord = coord.as_chunk_coord();

	let maptile = if let Ok(t) = map.get_tile(tile_coord) {
		t
	} else {
		return Err(());
	};
	let tile_entity = commands.entity(maptile.entity).id();
	if !tile_type.is_visible() {
		commands
			.entity(tile_entity)
			.remove::<WeightedTile>()
			.remove::<Tile>()
			.remove::<FallingTile>()
			.remove::<Collider>()
			.insert((SpriteBundle {
				texture: sprites.tile_outlines.get(39).unwrap().clone(), // use no outline sprite for tile sprite
				transform: Transform {
					translation: Vec3 {
						x: chunklocal_coord.x_f32() * tilesize_x_f32,
						y: chunklocal_coord.y_f32() * tilesize_y_f32,
						z: 0.0,
					},
					..Default::default()
				},
				..Default::default()
			},));
	} else {
		let texture_handle = sprites.tiles.get(&tile_type.get_sprite_dir_name()).unwrap();
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

		commands.entity(tile_entity).insert((
			Tile {
				tile_type,
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
		));
	}
	if tile_type.is_weighted() {
		commands.entity(tile_entity).insert(WeightedTile {
			granularity: tile_type.get_granularity(),
			liquid: tile_type.is_liquid(),
		});
	} else {
		commands.entity(tile_entity).remove::<WeightedTile>();
	}

	let new_maptile = MapTile {
		tile_type,
		..maptile
	};

	for x in -1..=1 {
		for y in -1..=1 {
			update_tile_event.send(UpdateTileEvent(
				tile_coord.moved(&Vec2::new(x as f32, y as f32)),
			));
		}
	}

	map.get_mut(&(chunk_coord.x_i32(), chunk_coord.y_i32()))
		.unwrap()
		.tiles
		.insert(
			(chunklocal_coord.x_u8(), chunklocal_coord.y_u8()),
			new_maptile,
		);

	Ok(new_maptile)
}
