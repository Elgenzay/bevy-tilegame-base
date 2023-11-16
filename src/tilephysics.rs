use std::{mem::discriminant, panic};

use bevy::{
	prelude::{
		App, Commands, Component, Entity, Event, EventReader, EventWriter, Last, Plugin,
		PostUpdate, Query, Res, ResMut, Transform, Update, Vec2, Vec3,
	},
	sprite::SpriteBundle,
};

use crate::{
	grid::{xorshift_from_coord, Coordinate, CreateTileEvent, Map, MapTile},
	light::{AddLightSourceEvent, LightingUpdateEvent},
	sprites::Sprites,
	tileoutline::ConnectedNeighbors,
	tiles::{set_tile, FallingTile, Tile, WeightedTile},
	tiletypes::{Liquid, LiquidInteraction, TileType},
	TickEvent,
};

const INITIAL_LIQUID_MOMENTUM: u8 = 200;

#[derive(Component)]
pub struct FlowingTile {
	x: i32,
}

pub struct TilePhysics;

impl Plugin for TilePhysics {
	fn build(&self, app: &mut App) {
		app.add_event::<UpdateTileEvent>()
			.add_event::<UpdateOutlineSpriteEvent>()
			.add_systems(Update, (flow_liquid_tile, apply_gravity))
			.add_systems(PostUpdate, update_tile)
			.add_systems(Last, update_outline_sprite_event);
	}
}

pub fn update_tile(
	mut map: ResMut<Map>,
	mut ev_update: EventReader<UpdateTileEvent>,
	mut ev_create_tile: EventWriter<CreateTileEvent>,
	mut commands: Commands,
	sprites: Res<Sprites>,
) {
	for ev in ev_update.read() {
		let tile = if let Some(t) = map.get_tile(ev.0) {
			t
		} else {
			continue;
		};
		if tile.tile_type.is_weighted() {
			commands
				.entity(tile.tile_entity)
				.insert(FallingTile { y: ev.0.y_i32() });
		}

		update_outline_sprite(tile, &mut commands, &sprites, &mut map);

		if let Ok(liquid) = tile.tile_type.get_liquid() {
			let new_sprite_override = if let Some(above) = map.get_tile(ev.0.moved(&Vec2::Y)) {
				above.tile_type.is_liquid()
					&& std::mem::discriminant(&above.tile_type)
						!= std::mem::discriminant(&tile.tile_type)
			} else {
				false
			};
			if new_sprite_override != liquid.sprite_override {
				ev_create_tile.send(CreateTileEvent::new(
					tile.tile_coord,
					tile.tile_type.with_liquid(Liquid {
						sprite_override: new_sprite_override,
						..liquid
					}),
					Some(tile),
				));
			}
		}
	}
}

fn update_outline_sprite_event(
	mut ev_update: EventReader<UpdateOutlineSpriteEvent>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	sprites: Res<Sprites>,
) {
	for ev in ev_update.read() {
		if let Some(maptile) = map.get_tile(ev.0) {
			update_outline_sprite(maptile, &mut commands, &sprites, &mut map);
		}
	}
}

fn update_outline_sprite(
	maptile: MapTile,
	commands: &mut Commands,
	sprites: &Sprites,
	map: &mut ResMut<Map>,
) {
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
					Some(t) => t.tile_type.is_solid(),
					None => false, // unloaded chunk
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

	if let Some(t) = map.get_tile_mut(maptile.tile_coord) {
		t.outline_id = outline_id;
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
	mut ev_addlightsource: EventWriter<AddLightSourceEvent>,
	mut ev_updatelighting: EventWriter<LightingUpdateEvent>,
) {
	for _ in tick.read() {
		let mut tuples = vec![];
		for (entity, tile, weighted_tile, falling_tile) in q_falling_tile.iter_mut() {
			tuples.push((entity, tile, weighted_tile, falling_tile.y));
		}
		if tuples.is_empty() {
			return;
		}
		tuples.sort_by(|a, b| a.3.cmp(&b.3));
		for tuple in tuples {
			let maptile = if let Some(t) = map.get_tile(tuple.1.coord) {
				t
			} else {
				continue;
			};
			let current_position = tuple.1.coord.as_tile_coord();
			match get_fall_coord(&map, current_position, tuple.2.granularity, maptile) {
				Ok(opt) => match opt {
					Some(coord) => {
						set_tile(
							&mut commands,
							current_position,
							TileType::Empty,
							&sprites,
							&mut map,
							&mut ev_updatetile,
							&mut ev_addlightsource,
							&mut ev_updatelighting,
						);
						set_tile(
							&mut commands,
							coord,
							tuple.1.tile_type,
							&sprites,
							&mut map,
							&mut ev_updatetile,
							&mut ev_addlightsource,
							&mut ev_updatelighting,
						);
					}
					None => {
						let mut cmds = commands.entity(tuple.0);
						cmds.remove::<FallingTile>();
						if maptile.tile_type.is_liquid() {
							cmds.insert(FlowingTile {
								x: current_position.x_i32(),
							});
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
	mut tick: EventReader<TickEvent>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	mut ev_updatetile: EventWriter<UpdateTileEvent>,
	sprites: Res<Sprites>,
	q_flowing_tiles: Query<(Entity, &Tile, &FlowingTile)>,
	mut ev_addlightsource: EventWriter<AddLightSourceEvent>,
	mut ev_updatelighting: EventWriter<LightingUpdateEvent>,
) {
	for t in tick.read() {
		let mut tuples = vec![];
		for (tile_entity, tile, flowing_tile) in q_flowing_tiles.iter() {
			tuples.push((tile_entity, tile, flowing_tile.x));
		}
		if t.0 % 2 == 0 {
			tuples.sort_by(|a, b| a.2.cmp(&b.2));
		} else {
			tuples.sort_by(|a, b| b.2.cmp(&a.2));
		}
		'outer: for tuple in tuples {
			let tile_entity = tuple.0;
			let tile = tuple.1;
			let maptile = if let Some(t) = map.get_tile(tile.coord) {
				if t.tile_type.is_liquid() {
					t
				} else {
					continue;
				}
			} else {
				continue;
			};
			let fluidity = maptile.tile_type.get_fluidity();
			let maptile_liquid = if let Ok(v) = maptile.tile_type.get_liquid() {
				v
			} else {
				commands.entity(tile_entity).remove::<FlowingTile>();
				continue;
			};
			let rand_bool = xorshift_from_coord(maptile.tile_coord) % 2 == 0;
			let mut left_blocked = false;
			let mut right_blocked = false;
			for i in 0..=(fluidity * fluidity) as i32 {
				for m in if rand_bool { [-1, 1] } else { [1, -1] } {
					if (m == -1 && left_blocked) || (m == 1 && right_blocked) {
						continue;
					}

					let x = i * m;
					if i == 0 && m == -1 {
						continue;
					}
					let below_coord = maptile.tile_coord.moved(&Vec2::new(x as f32, -1.0));
					if let Some(t) = map.get_tile(below_coord) {
						if discriminant(&t.tile_type) == discriminant(&maptile.tile_type) {
							let other_level = t.tile_type.liquid().level;
							let other_emptiness = u8::MAX - other_level;
							if other_emptiness != 0 {
								let this_level = maptile_liquid.level;
								let this_remainder =
									(other_level as i32 + this_level as i32) - u8::MAX as i32;
								let (new_level, new_other_level) = if this_remainder < 0 {
									(0, other_level + this_level)
								} else {
									(this_remainder as u8, u8::MAX)
								};
								set_tile(
									&mut commands,
									maptile.tile_coord,
									if new_level != 0 {
										maptile.tile_type.with_liquid(Liquid {
											level: new_level,
											..Default::default()
										})
									} else {
										TileType::Empty
									},
									&sprites,
									&mut map,
									&mut ev_updatetile,
									&mut ev_addlightsource,
									&mut ev_updatelighting,
								);
								set_tile(
									&mut commands,
									below_coord,
									maptile.tile_type.with_liquid(Liquid {
										level: new_other_level,
										..Default::default()
									}),
									&sprites,
									&mut map,
									&mut ev_updatetile,
									&mut ev_addlightsource,
									&mut ev_updatelighting,
								);
								continue 'outer;
							}
						} else {
							if m == -1 {
								left_blocked = true;
							} else {
								right_blocked = true;
							}
							if let Ok(t_liquid) = t.tile_type.get_liquid() {
								let mut cont = true;
								match maptile.tile_type.get_liquid_interaction_with(t.tile_type) {
									LiquidInteraction::Vaporize => {
										set_tile(
											&mut commands,
											maptile.tile_coord,
											TileType::Empty,
											&sprites,
											&mut map,
											&mut ev_updatetile,
											&mut ev_addlightsource,
											&mut ev_updatelighting,
										);
										set_tile(
											&mut commands,
											below_coord,
											maptile.tile_type,
											&sprites,
											&mut map,
											&mut ev_updatetile,
											&mut ev_addlightsource,
											&mut ev_updatelighting,
										);
									}
									LiquidInteraction::Vaporized => {
										set_tile(
											&mut commands,
											maptile.tile_coord,
											TileType::Empty,
											&sprites,
											&mut map,
											&mut ev_updatetile,
											&mut ev_addlightsource,
											&mut ev_updatelighting,
										);
									}
									LiquidInteraction::Float => {
										cont = false;
										if !t_liquid.sprite_override {
											set_tile(
												&mut commands,
												t.tile_coord,
												t.tile_type.with_liquid(Liquid {
													sprite_override: true,
													..t_liquid
												}),
												&sprites,
												&mut map,
												&mut ev_updatetile,
												&mut ev_addlightsource,
												&mut ev_updatelighting,
											);
										}
									}
									LiquidInteraction::Sink => {
										set_tile(
											&mut commands,
											maptile.tile_coord,
											t.tile_type,
											&sprites,
											&mut map,
											&mut ev_updatetile,
											&mut ev_addlightsource,
											&mut ev_updatelighting,
										);
										set_tile(
											&mut commands,
											below_coord,
											maptile.tile_type,
											&sprites,
											&mut map,
											&mut ev_updatetile,
											&mut ev_addlightsource,
											&mut ev_updatelighting,
										);
									}
								}
								if cont {
									continue 'outer;
								}
							}
						}
					}
				}
			}
			if fluidity < 10 && t.0 % (11 - fluidity as u64) != 0 {
				continue;
			}
			let get_level = |coord| {
				if let Some(t) = map.get_tile(coord) {
					if discriminant(&t.tile_type) == discriminant(&maptile.tile_type) {
						t.tile_type.liquid().level as i32 // existing liquid of same type
					} else if !t.tile_type.is_solid() {
						if t.tile_type.is_liquid() {
							match maptile.tile_type.get_liquid_interaction_with(t.tile_type) {
								LiquidInteraction::Vaporize => 0_i32,
								LiquidInteraction::Vaporized => -1_i32,
								LiquidInteraction::Float => -1_i32,
								LiquidInteraction::Sink => -1_i32,
							}
						} else {
							0_i32 // can flow
						}
					} else {
						-1_i32 // blocked by solid
					}
				} else {
					-1_i32 // blocked by unloaded chunk
				}
			};

			let left_coord = maptile.tile_coord.moved(&Vec2::NEG_X);
			let right_coord = maptile.tile_coord.moved(&Vec2::X);
			let mut left_level = get_level(left_coord);
			let mut right_level = get_level(right_coord);
			let left_level_initial = left_level;
			let right_level_initial = right_level;
			let mut this_level = maptile_liquid.level as i32;
			let this_level_initial = maptile_liquid.level as i32;
			let mut significant = false;
			for lvl in [left_level_initial, right_level_initial] {
				if lvl != -1 && (this_level - lvl).abs() > 1 {
					significant = true;
					break;
				}
			}

			let (mut flow_right, momentum) = if let Some(v) = maptile_liquid.flowing_right {
				(
					v,
					if significant {
						INITIAL_LIQUID_MOMENTUM
					} else {
						maptile_liquid.momentum - 1
					},
				)
			} else {
				(
					rand_bool,
					if significant {
						INITIAL_LIQUID_MOMENTUM
					} else {
						1
					},
				)
			};

			let stagnant = momentum <= 1;
			if stagnant {
				commands.entity(tile_entity).remove::<FlowingTile>();
			} else {
				loop {
					if this_level == 0 {
						break;
					}

					let left_blocked = left_level == -1 || left_level >= this_level;
					let right_blocked = right_level == -1 || right_level >= this_level;

					flow_right = if right_blocked {
						if left_blocked {
							break;
						} else {
							false
						}
					} else if left_blocked {
						true
					} else {
						flow_right
					};

					if flow_right {
						right_level += 1;
					} else {
						left_level += 1;
					}
					this_level -= 1;
					flow_right = !flow_right;
				}
			}

			let mut set_liquid = |flow_right: bool, level: i32, level_initial, coord| match level {
				_ if level > 0 => {
					if level != level_initial {
						let new_tile = maptile.tile_type.with_liquid(Liquid {
							level: level as u8,
							flowing_right: if stagnant { None } else { Some(!flow_right) },
							momentum: if stagnant { 0 } else { momentum },
							..maptile_liquid
						});
						set_tile(
							&mut commands,
							coord,
							new_tile,
							&sprites,
							&mut map,
							&mut ev_updatetile,
							&mut ev_addlightsource,
							&mut ev_updatelighting,
						);
					}
				}
				0 => {
					set_tile(
						&mut commands,
						coord,
						TileType::Empty,
						&sprites,
						&mut map,
						&mut ev_updatetile,
						&mut ev_addlightsource,
						&mut ev_updatelighting,
					);
				}
				_ => (),
			};
			set_liquid(flow_right, left_level, left_level_initial, left_coord);
			set_liquid(flow_right, right_level, right_level_initial, right_coord);
			set_liquid(
				flow_right,
				this_level,
				this_level_initial,
				maptile.tile_coord,
			);
		}
	}
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
					Some(t) => {
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
					None => return Err(()),
				}
			}
			if (left_blocked && m == -1) || (right_blocked && m == 1) {
				continue;
			}
			let new_coord = current_position.moved(&Vec2::new(x_f32, -1.0));
			match map.get_tile(new_coord) {
				Some(t) => {
					if !maptile.tile_type.is_obstructed_by(t.tile_type) {
						return Ok(Some(new_coord));
					} else {
						continue;
					}
				}
				None => return Err(()),
			}
		}
		if left_blocked && right_blocked {
			return Ok(None);
		}
	}
	Ok(None)
}

#[derive(Event)]
pub struct UpdateTileEvent(pub Coordinate);

#[derive(Event)]
pub struct UpdateOutlineSpriteEvent(pub Coordinate);
