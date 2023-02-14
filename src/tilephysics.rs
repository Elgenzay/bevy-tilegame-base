use bevy::{
	prelude::{App, Commands, Entity, EventReader, EventWriter, Plugin, Query, Res, ResMut, Vec2},
	time::Time,
};

use crate::{
	grid::{Coordinate, Map},
	sprites::Sprites,
	tiles::{create_tile_entity, FallingTile, Tile, WeightedTile},
	TickTimer,
};

pub struct TilePhysics;

impl Plugin for TilePhysics {
	fn build(&self, app: &mut App) {
		app.add_system(apply_gravity)
			.add_event::<UpdateTilePhysicsEvent>()
			.add_system(update_tile_physics);
	}
}

pub fn update_tile_physics(
	map: Res<Map>,
	mut ev_update: EventReader<UpdateTilePhysicsEvent>,
	mut commands: Commands,
	q_tiles: Query<&Tile>,
) {
	for ev in ev_update.iter() {
		let tiles = match ev.0 {
			Coordinate::Chunk { x: _, y: _ } => match map.get_tiles(ev.0) {
				Ok(v) => v,
				Err(_) => continue,
			},
			Coordinate::ChunkLocal { x: _, y: _ } => {
				panic!("ChunkLocal coordinate passed to UpdateTilePhysicsEvent")
			}
			Coordinate::Tile { x: _, y: _ } | Coordinate::World { x: _, y: _ } => {
				match map.get_tile(ev.0) {
					Ok(opt) => match opt {
						Some(tile) => vec![tile],
						None => continue,
					},
					Err(_) => continue,
				}
			}
		};
		for tile_parent in tiles {
			let tile = match q_tiles.get(*tile_parent) {
				Ok(v) => v,
				Err(_) => continue,
			};
			if tile.tile_type.is_weighted() {
				commands
					.entity(*tile_parent)
					.insert(FallingTile(tile.coordinate.as_tile_coord().y_i32()));
			}
		}
	}
}

fn apply_gravity(
	mut q_falling_tile: Query<(Entity, &Tile, &WeightedTile, &FallingTile)>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	time: Res<Time>,
	mut timer: ResMut<TickTimer>,
	mut ev_updatetile: EventWriter<UpdateTilePhysicsEvent>,
	sprites: Res<Sprites>,
) {
	if !timer.0.tick(time.delta()).just_finished() {
		return;
	}
	let mut tuples = vec![];
	for (entity, tile, weighted_tile, falling_tile) in q_falling_tile.iter_mut() {
		tuples.push((entity, tile, weighted_tile, falling_tile.0));
	}
	if tuples.len() == 0 {
		return;
	}
	tuples.sort_by(|a, b| a.3.cmp(&b.3));
	for tuple in tuples {
		let current_position = tuple.1.coordinate.as_tile_coord();
		match get_fall_coord(map.as_ref(), current_position, tuple.2.granularity) {
			Ok(opt) => match opt {
				Some(coord) => {
					let e =
						create_tile_entity(&mut commands, coord, tuple.1.tile_type, &sprites, &map);
					if let Err(_) = map.set_tile(&mut commands, current_position, None) {
						continue;
						//unloaded chunk
					};
					commands.entity(e).insert(FallingTile(coord.y_i32()));
					if let Err(_) = map.set_tile(&mut commands, coord, Some(e)) {
						continue;
						//unloaded chunk
					};
					for c in current_position.get_neighboring(1) {
						ev_updatetile.send(UpdateTilePhysicsEvent(c));
					}
				}
				None => {
					let _ = &commands.entity(tuple.0).remove::<FallingTile>();
					continue;
				}
			},
			Err(()) => continue, //unloaded chunk
		};
	}
}

fn get_fall_coord(
	map: &Map,
	current_position: Coordinate,
	granularity: u8,
) -> Result<Option<Coordinate>, ()> {
	let current_position = current_position.as_tile_coord();
	for x_abs in 0..=granularity {
		let mut left_blocked = false;
		let mut right_blocked = false;
		for m in [-1, 1] {
			let x_i8 = x_abs as i8 * m as i8;
			let x_f32 = x_i8 as f32;
			if x_i8 != 0 {
				match map.get_tile(current_position.moved(&Vec2::new(x_f32, 0.0))) {
					Ok(opt) => match opt {
						Some(_) => {
							if m == -1 {
								left_blocked = true;
							} else {
								right_blocked = true;
							}
						}
						None => (),
					},
					Err(()) => return Err(()),
				}
			}
			let new_coord = current_position.moved(&Vec2::new(x_f32, -1.0));
			match map.get_tile(new_coord) {
				Ok(opt) => match opt {
					Some(_) => continue,
					None => return Ok(Some(new_coord)),
				},
				Err(()) => return Err(()),
			}
		}
		if left_blocked && right_blocked {
			return Ok(None);
		}
	}
	Ok(None)
}

pub struct UpdateTilePhysicsEvent(pub Coordinate);
