use std::panic;

use bevy::{
	prelude::{
		App, Commands, CoreStage, Entity, EventReader, EventWriter, Plugin, Query, Res, ResMut,
		Transform, Vec2, Vec3,
	},
	sprite::SpriteBundle,
	time::Time,
};

use crate::{
	grid::{Coordinate, Map, MapTile},
	sprites::Sprites,
	tileoutline::ConnectedNeighbors,
	tiles::{set_tile, FallingTile, Tile, WeightedTile},
	tiletypes::TileType,
	TickTimer,
};

pub struct TilePhysics;

impl Plugin for TilePhysics {
	fn build(&self, app: &mut App) {
		app.add_system(apply_gravity)
			.add_event::<UpdateTileEvent>()
			.add_system_to_stage(CoreStage::Last, update_tile);
	}
}

pub fn update_tile(
	map: Res<Map>,
	mut ev_update: EventReader<UpdateTileEvent>,
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
			if let Some(mut e) = commands.get_entity(tile.entity) {
				e.insert(FallingTile(ev.0.y_i32()));
			}
		}
		update_outline_sprite(ev.0, &map, &mut commands, tile, &sprites)
	}
}

fn update_outline_sprite(
	coordinate: Coordinate,
	map: &Map,
	commands: &mut Commands,
	maptile: MapTile,
	sprites: &Sprites,
) {
	let outline_id = if !maptile.tile_type.is_visible() {
		40 // no outline
	} else {
		let mut connected = ConnectedNeighbors::new();
		for x in -1..=1 {
			for y in -1..=1 {
				if x == 0 && y == 0 {
					continue;
				}
				let tile = map.get_tile(
					coordinate
						.as_tile_coord()
						.moved(&Vec2::new(x as f32, y as f32)),
				);
				if match tile {
					Ok(t) => t.tile_type.is_solid(),
					Err(_) => false, //unloaded chunk
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

	if let Some(mut e) = commands.get_entity(maptile.outline) {
		e.insert(SpriteBundle {
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
}

fn apply_gravity(
	mut q_falling_tile: Query<(Entity, &Tile, &WeightedTile, &FallingTile)>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	time: Res<Time>,
	mut timer: ResMut<TickTimer>,
	mut ev_updatetile: EventWriter<UpdateTileEvent>,
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
		match get_fall_coord(&map, current_position, tuple.2.granularity) {
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
					let tile = if let Ok(t) = set_tile(
						&mut commands,
						coord,
						tuple.1.tile_type,
						&sprites,
						&mut map,
						&mut ev_updatetile,
					) {
						t
					} else {
						continue; //unloaded chunk
					};
					if let Some(mut e) = commands.get_entity(tile.entity) {
						e.insert(FallingTile(coord.y_i32()));
					}
				}
				None => {
					if let Some(mut e) = commands.get_entity(tuple.0) {
						e.remove::<FallingTile>();
					}
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
					Ok(t) => {
						if t.tile_type.is_solid() {
							if m == -1 {
								left_blocked = true;
							} else {
								right_blocked = true;
							}
						}
					}
					Err(()) => return Err(()),
				}
			}
			let new_coord = current_position.moved(&Vec2::new(x_f32, -1.0));
			match map.get_tile(new_coord) {
				Ok(t) => {
					if !t.tile_type.is_solid() {
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
