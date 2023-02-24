use bevy::{
	prelude::{Commands, Component, EventWriter, Quat, Transform, Vec2, Vec3},
	sprite::SpriteBundle,
};

use crate::{
	grid::{Coordinate, Map, MapTile},
	playerphysics::Collider,
	sprites::Sprites,
	tilephysics::UpdateTileEvent,
	tiletypes::TileType,
};

#[derive(Component)]
pub struct Tile {
	pub tile_type: TileType,
	pub coord: Coordinate,
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
	let tile_coord = coord.as_tile_coord();
	let chunklocal_coord = coord.as_chunklocal_coord();
	let chunk_coord = coord.as_chunk_coord();

	let maptile = if let Ok(t) = map.get_tile(tile_coord) {
		t
	} else {
		return Err(());
	};
	if !tile_type.is_visible() {
		commands
			.entity(maptile.tile_entity)
			.remove::<WeightedTile>()
			.remove::<Tile>()
			.remove::<FallingTile>()
			.remove::<Collider>();
		commands
			.entity(maptile.sprite_entity)
			.insert((SpriteBundle {
				texture: sprites.tile_outlines.get(39).unwrap().clone(), // use no outline sprite for tile sprite
				transform: Transform {
					translation: Vec3::ZERO,
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
		let (rot, scale) = if tile_type.morph_sprite() {
			(
				(i % 4) as f32 * 1.5708,
				match (i / 100) % 3 {
					1 => (1.0, -1.0), // flip vertical
					2 => (-1.0, 1.0), // flip horizontal
					_ => (1.0, 1.0),  // no flip
				},
			)
		} else {
			(0.0, (1.0, 1.0))
		};

		i = i / 10;
		i = i.abs() % texture_handle.len() as i32;

		commands.entity(maptile.tile_entity).insert((
			Tile {
				tile_type,
				coord: tile_coord,
			},
			Collider,
		));
		commands.entity(maptile.sprite_entity).insert(SpriteBundle {
			texture: texture_handle.get(i as usize).unwrap().clone(),
			transform: Transform {
				rotation: Quat::from_rotation_z(rot),
				scale: Vec3::new(scale.0, scale.1, 1.0),
				translation: Vec3::ZERO,
			},
			..Default::default()
		});
	}
	if tile_type.is_weighted() {
		commands.entity(maptile.tile_entity).insert(WeightedTile {
			granularity: tile_type.get_granularity(),
			liquid: tile_type.is_liquid(),
		});
	} else {
		commands
			.entity(maptile.tile_entity)
			.remove::<WeightedTile>();
	}

	let new_maptile = MapTile {
		tile_type,
		outline_id: 50,
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
