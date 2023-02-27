use std::{mem::discriminant, panic};

use bevy::{
	prelude::{
		App, Commands, CoreStage, Entity, EventReader, EventWriter, Plugin, Query, Res, ResMut,
		Transform, Vec2, Vec3,
	},
	sprite::SpriteBundle,
};

use crate::{
	grid::{xorshift_from_coord, Coordinate, Map, MapTile},
	sprites::Sprites,
	tileoutline::ConnectedNeighbors,
	tiles::{set_tile, FallingTile, Tile, WeightedTile},
	tiletypes::TileType,
	TickEvent, FLUID_PER_TILE,
};

pub struct TilePhysics;

impl Plugin for TilePhysics {
	fn build(&self, app: &mut App) {
		app.add_system(apply_gravity)
			.add_event::<UpdateTileEvent>()
			.add_event::<UpdateOutlineSpriteEvent>()
			.add_system_to_stage(CoreStage::PostUpdate, update_outline_sprite_event)
			.add_system_to_stage(CoreStage::PreUpdate, update_tile);
	}
}

pub fn update_tile(
	map: Res<Map>,
	mut ev_update: EventReader<UpdateTileEvent>,
	//mut ev_update_outline: EventWriter<UpdateOutlineSpriteEvent>,
	mut commands: Commands,
	sprites: Res<Sprites>,
) {
	for ev in ev_update.iter() {
		let tile = if let Ok(t) = map.get_tile(ev.0) {
			t
		} else {
			continue;
		};
		if tile.tile_type.is_weighted() {
			commands
				.entity(tile.tile_entity)
				.insert(FallingTile { y: ev.0.y_i32() });
		}
		//ev_update_outline.send(UpdateOutlineSpriteEvent(ev.0));
		update_outline_sprite(tile, &mut commands, &sprites, &map);
	}
}

fn update_outline_sprite_event(
	mut ev_update: EventReader<UpdateOutlineSpriteEvent>,
	map: Res<Map>,
	mut commands: Commands,
	sprites: Res<Sprites>,
) {
	for ev in ev_update.iter() {
		if let Ok(maptile) = map.get_tile(ev.0) {
			update_outline_sprite(maptile, &mut commands, &sprites, &map);
		}
	}
}

fn update_outline_sprite(maptile: MapTile, commands: &mut Commands, sprites: &Sprites, map: &Map) {
	let outline_id = if !maptile.tile_type.is_visible() || maptile.tile_type.is_liquid() {
		40 // no outline
	} else {
		let mut connected = ConnectedNeighbors::new();
		for x in -1..=1 {
			for y in -1..=1 {
				if x == 0 && y == 0 {
					continue;
				}
				let tile = map.get_tile(maptile.tile_coord.moved(&Vec2::new(x as f32, y as f32)));
				if match tile {
					Ok(t) => t.tile_type.is_solid(),
					Err(_) => false, // unloaded chunk
				} {
					match x {
						-1 => match y {
							-1 => connected.bottom_left = true,
							0 => connected.left = true,
							1 => connected.top_left = true,
							_ => panic!(),
						},
						0 => match y {
							-1 => connected.bottom = true,
							1 => connected.top = true,
							_ => panic!(),
						},
						1 => match y {
							-1 => connected.bottom_right = true,
							0 => connected.right = true,
							1 => connected.top_right = true,
							_ => panic!(),
						},
						_ => panic!(),
					}
				}
			}
		}
		connected.get_outline_id()
	};

	if maptile.outline_id == outline_id {
		return;
	}
	commands
		.entity(maptile.outline_entity)
		.insert(SpriteBundle {
			texture: sprites.tile_outlines.get(outline_id - 1).unwrap().clone(),
			transform: Transform {
				translation: Vec3 {
					x: 0.0,
					y: 0.0,
					z: 1.0,
				},
				..Default::default()
			},
			..Default::default()
		});
}

fn apply_gravity(
	mut q_falling_tile: Query<(Entity, &Tile, &WeightedTile, &FallingTile)>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	mut ev_updatetile: EventWriter<UpdateTileEvent>,
	mut tick: EventReader<TickEvent>,
	sprites: Res<Sprites>,
) {
	for _ in tick.iter() {
		let mut tuples = vec![];
		for (entity, tile, weighted_tile, falling_tile) in q_falling_tile.iter_mut() {
			tuples.push((entity, tile, weighted_tile, falling_tile.y));
		}
		if tuples.len() == 0 {
			return;
		}
		tuples.sort_by(|a, b| a.3.cmp(&b.3));
		for tuple in tuples {
			let maptile = if let Ok(t) = map.get_tile(tuple.1.coord) {
				t
			} else {
				continue;
			};
			let current_position = tuple.1.coord.as_tile_coord();
			match get_fall_coord(&map, current_position, tuple.2.granularity, maptile) {
				Ok(opt) => match opt {
					Some(coord) => {
						let _ = set_tile(
							&mut commands,
							current_position,
							TileType::Empty,
							&sprites,
							&mut map,
							&mut ev_updatetile,
						);
						let _ = set_tile(
							&mut commands,
							coord,
							tuple.1.tile_type,
							&sprites,
							&mut map,
							&mut ev_updatetile,
						);
					}
					None => {
						commands.entity(tuple.0).remove::<FallingTile>();
						if maptile.tile_type.is_liquid() {
							flow_liquid_tile(
								&mut map,
								maptile,
								&mut commands,
								&mut ev_updatetile,
								&sprites,
							);
						}
						continue;
					}
				},
				Err(()) => continue, // unloaded chunk
			};
		}
	}
}

fn flow_liquid_tile(
	map: &mut Map,
	maptile: MapTile,
	commands: &mut Commands,
	ev_updatetile: &mut EventWriter<UpdateTileEvent>,
	sprites: &Sprites,
) {
	let below_coord = maptile.tile_coord.moved(&Vec2::NEG_Y);
	if let Ok(t) = map.get_tile(below_coord) {
		if discriminant(&t.tile_type) == discriminant(&maptile.tile_type) {
			let other_level = t.tile_type.get_liquid_level();
			let other_emptiness = FLUID_PER_TILE - other_level;
			if other_emptiness != 0 {
				let this_level = maptile.tile_type.get_liquid_level();
				let this_remainder = other_level as i8 + this_level as i8 - FLUID_PER_TILE as i8;
				let (new_level, new_other_level) = if this_remainder < 0 {
					(0, other_level + this_level)
				} else {
					(this_remainder as u8, FLUID_PER_TILE)
				};
				let _ = set_tile(
					commands,
					maptile.tile_coord,
					if new_level != 0 {
						maptile.tile_type.with_liquid_level(new_level)
					} else {
						TileType::Empty
					},
					sprites,
					map,
					ev_updatetile,
				);
				let _ = set_tile(
					commands,
					below_coord,
					maptile.tile_type.with_liquid_level(new_other_level),
					sprites,
					map,
					ev_updatetile,
				);
				return;
			}
		}
	}
	if maptile.tile_type.get_liquid_level() == 1 {
		return;
	}

	let get_level = |coord| {
		return if let Ok(t) = map.get_tile(coord) {
			if discriminant(&t.tile_type) == discriminant(&maptile.tile_type) {
				t.tile_type.get_liquid_level() as i8 // existing liquid of same type
			} else if !t.tile_type.is_solid() {
				0 as i8 // can flow. interactions with other liquids go here
			} else {
				-1 as i8 // blocked by solid
			}
		} else {
			-1 as i8 // unloaded chunk
		};
	};

	let left_coord = maptile.tile_coord.moved(&Vec2::NEG_X);
	let right_coord = maptile.tile_coord.moved(&Vec2::X);
	let mut left_level = get_level(left_coord);
	let mut right_level = get_level(right_coord);
	if left_level == -1 && right_level == -1 {
		return;
	}
	let left_level_initial = left_level;
	let right_level_initial = right_level;
	let mut this_level = maptile.tile_type.get_liquid_level() as i8;
	let this_level_initial = maptile.tile_type.get_liquid_level() as i8;
	let mut flow_left = xorshift_from_coord(maptile.tile_coord) % 2 == 0;
	loop {
		flow_left = !flow_left;
		if this_level == 1 {
			break;
		}

		let left_blocked = if left_level == -1 || left_level >= this_level {
			true
		} else {
			false
		};
		let right_blocked = if right_level == -1 || right_level >= this_level {
			true
		} else {
			false
		};

		flow_left = if left_blocked {
			if right_blocked {
				break;
			} else {
				false
			}
		} else {
			if right_blocked {
				true
			} else {
				flow_left
			}
		};

		if flow_left {
			left_level += 1;
		} else {
			right_level += 1;
		}
		this_level -= 1;
	}
	let mut set_liquid = |level: i8, level_initial, coord| {
		if level > 0 {
			if level != level_initial || level != FLUID_PER_TILE as i8 {
				let _ = set_tile(
					commands,
					coord,
					maptile.tile_type.with_liquid_level(level as u8),
					sprites,
					map,
					ev_updatetile,
				);
			}
		}
	};
	set_liquid(left_level, left_level_initial, left_coord);
	set_liquid(right_level, right_level_initial, right_coord);
	set_liquid(this_level, this_level_initial, maptile.tile_coord);
}

fn get_fall_coord(
	map: &Map,
	current_position: Coordinate,
	granularity: u8,
	maptile: MapTile,
) -> Result<Option<Coordinate>, ()> {
	let current_position = current_position.as_tile_coord();
	let mut left_blocked = false;
	let mut right_blocked = false;
	let directions = if xorshift_from_coord(current_position) % 2 == 0 {
		[-1, 1]
	} else {
		[1, -1]
	};
	'outer: for x_abs in 0..=granularity {
		for m in directions {
			if (left_blocked && m == -1) || (right_blocked && m == 1) {
				continue;
			}
			let x_i8 = x_abs as i8 * m as i8;
			let x_f32 = x_i8 as f32;
			if x_i8 != 0 {
				match map.get_tile(current_position.moved(&Vec2::new(x_f32, 0.0))) {
					Ok(t) => {
						if maptile.tile_type.is_obstructed_by(t.tile_type) {
							if m == -1 {
								left_blocked = true;
							} else {
								right_blocked = true;
							}
							if left_blocked && right_blocked {
								break 'outer;
							}
						}
					}
					Err(()) => return Err(()),
				}
			}
			if (left_blocked && m == -1) || (right_blocked && m == 1) {
				continue;
			}
			let new_coord = current_position.moved(&Vec2::new(x_f32, -1.0));
			match map.get_tile(new_coord) {
				Ok(t) => {
					if !maptile.tile_type.is_obstructed_by(t.tile_type) {
						return Ok(Some(new_coord));
					} else {
						continue;
					}
				}
				Err(()) => return Err(()),
			}
		}
		if left_blocked && right_blocked {
			return Ok(None);
		}
	}
	Ok(None)
}

pub struct UpdateTileEvent(pub Coordinate);
pub struct UpdateOutlineSpriteEvent(pub Coordinate);
