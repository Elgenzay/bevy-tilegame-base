use bevy::{
	prelude::{
		App, Commands, CoreStage, Entity, EventReader, EventWriter, Plugin, Query, Res, ResMut,
		Transform, Vec2, Vec3,
	},
	sprite::SpriteBundle,
	time::Time,
};

use crate::{
	grid::{destroy_tile, Coordinate, Map, MapTile},
	sprites::Sprites,
	tileoutline::ConnectedNeighbors,
	tiles::{create_tile, FallingTile, Tile, WeightedTile},
	TickTimer,
};

pub struct TilePhysics;

impl Plugin for TilePhysics {
	fn build(&self, app: &mut App) {
		app.add_system(apply_gravity)
			.add_event::<UpdateTileEvent>()
			.add_system_to_stage(CoreStage::PostUpdate, update_tile);
	}
}

pub fn update_tile(
	map: Res<Map>,
	mut ev_update: EventReader<UpdateTileEvent>,
	mut commands: Commands,
	q_tiles: Query<&Tile>,
	sprites: Res<Sprites>,
) {
	for ev in ev_update.iter() {
		let tile = match q_tiles.get(ev.0.entity) {
			Ok(v) => v,
			Err(_) => continue,
		};
		let tile_coord = tile.coordinate.as_tile_coord();
		if tile.tile_type.is_weighted() {
			if let Some(mut e) = commands.get_entity(ev.0.entity) {
				e.insert(FallingTile(tile_coord.y_i32()));
			}
		}
		update_sprite(tile_coord, &map, &mut commands, ev.0, &sprites)
	}
}

fn update_sprite(
	coordinate: Coordinate,
	map: &Map,
	commands: &mut Commands,
	maptile: MapTile,
	sprites: &Sprites,
) {
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
				Ok(opt) => match opt {
					Some(_) => true,
					None => false,
				},
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
	if let Some(mut e) = commands.get_entity(maptile.outline) {
		e.insert(SpriteBundle {
			texture: sprites
				.tile_outlines
				.get(connected.get_outline_index() - 1)
				.unwrap()
				.clone(),
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
					destroy_tile(
						current_position,
						&mut map,
						&mut commands,
						&mut ev_updatetile,
					);
					let tile = if let Ok(t) = create_tile(
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

pub struct UpdateTileEvent(pub MapTile);
