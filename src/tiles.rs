use bevy::{
	prelude::{Commands, Component, EventWriter, Quat, Transform, Vec2, Vec3},
	sprite::SpriteBundle,
};

use crate::{
	grid::{xorshift_from_coord, Coordinate, Map, MapTile},
	light::{AddLightSourceEvent, LightingUpdateEvent},
	playerphysics::Collider,
	sprites::Sprites,
	tilephysics::{FlowingTile, UpdateTileEvent},
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
}

#[derive(Component, Ord, Eq, PartialEq, PartialOrd)]
pub struct FallingTile {
	pub y: i32,
}

pub fn set_tile(
	commands: &mut Commands,
	coord: Coordinate,
	tile_type: TileType,
	sprites: &Sprites,
	map: &mut Map,
	update_tile_event: &mut EventWriter<UpdateTileEvent>,
	event_add_lightsource: &mut EventWriter<AddLightSourceEvent>,
	event_update_lighting: &mut EventWriter<LightingUpdateEvent>,
) {
	let _ = set_tile_result(
		commands,
		coord,
		tile_type,
		sprites,
		map,
		update_tile_event,
		event_add_lightsource,
		event_update_lighting,
	);
}

pub fn set_tile_result(
	commands: &mut Commands,
	coord: Coordinate,
	tile_type: TileType,
	sprites: &Sprites,
	map: &mut Map,
	update_tile_event: &mut EventWriter<UpdateTileEvent>,
	event_add_lightsource: &mut EventWriter<AddLightSourceEvent>,
	event_update_lighting: &mut EventWriter<LightingUpdateEvent>,
) -> Result<MapTile, ()> {
	let tile_coord = coord.as_tile_coord();
	let chunklocal_coord = coord.as_chunklocal_coord();
	let chunk_coord = coord.as_chunk_coord();

	let maptile = if let Some(t) = map.get_tile(tile_coord) {
		t
	} else {
		return Err(());
	};

	commands
		.entity(maptile.tile_entity)
		.remove::<FallingTile>()
		.remove::<FlowingTile>();

	if !tile_type.is_visible() {
		commands.entity(maptile.tile_entity).remove::<Tile>();
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
	} else if !tile_type.is_liquid() {
		let texture_handle = sprites.tiles.get(&tile_type.get_sprite_dir_name()).unwrap();

		let mut i = xorshift_from_coord(tile_coord);

		let (rot, scale) = if tile_type.morph_sprite() {
			(
				(i % 4) as f32 * std::f32::consts::FRAC_PI_2,
				match (i / 10) % 3 {
					1 => (1.0, -1.0), // flip vertical
					2 => (-1.0, 1.0), // flip horizontal
					_ => (1.0, 1.0),  // no flip
				},
			)
		} else {
			(0.0, (1.0, 1.0))
		};
		i = i.abs() % texture_handle.len() as i32;

		commands.entity(maptile.sprite_entity).insert(SpriteBundle {
			texture: texture_handle.get(i as usize).unwrap().clone(),
			transform: Transform {
				rotation: Quat::from_rotation_z(rot),
				scale: Vec3::new(scale.0, scale.1, 1.0),
				translation: Vec3::ZERO,
			},
			..Default::default()
		});
	} else {
		let texture_handle = sprites.tiles.get(&tile_type.get_sprite_dir_name()).unwrap();
		let i = if tile_type.liquid().sprite_override {
			texture_handle.len() - 1
		} else {
			((texture_handle.len() as f32 - 1.0)
				* (tile_type.liquid().level as f32 / u8::MAX as f32)) as usize
		};
		let t = texture_handle.get(i).expect(
			&format!(
				"Missing liquid tile texture: {} index {}",
				tile_type.get_name(),
				i,
			)[..],
		);
		commands.entity(maptile.sprite_entity).insert(SpriteBundle {
			texture: t.clone(),
			..Default::default()
		});
	}

	let mut cmds = commands.entity(maptile.tile_entity);

	cmds.insert(Tile {
		tile_type,
		coord: tile_coord,
	});

	if tile_type.is_solid() {
		cmds.insert(Collider);
	} else {
		cmds.remove::<Collider>();
	}

	if tile_type.is_weighted() {
		cmds.insert(WeightedTile {
			granularity: tile_type.get_granularity(),
		});
	} else {
		cmds.remove::<WeightedTile>();
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

	let maptile_mut = map
		.get_mut(&(chunk_coord.x_i32(), chunk_coord.y_i32()))
		.unwrap()
		.tiles
		.get_mut(&(chunklocal_coord.x_u8(), chunklocal_coord.y_u8()));
	if let Some(v) = maptile_mut {
		if v.tile_type.is_emitter() {
			// todo remove lightsource
		}
		if new_maptile.tile_type.is_emitter() {
			event_add_lightsource.send(AddLightSourceEvent(new_maptile));
		}
		if v.tile_type.is_opaque() != new_maptile.tile_type.is_opaque() {
			event_update_lighting.send(LightingUpdateEvent(v.tile_coord));
		}
		*v = new_maptile;
	}

	Ok(new_maptile)
}
