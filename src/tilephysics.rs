use bevy::{
	prelude::{
		App, AssetServer, Commands, Entity, EventReader, Plugin, Query, Res, ResMut, Vec2, With,
	},
	time::Time,
};

use crate::{
	grid::{Coordinate, Map},
	tiles::{create_tile_entity, FallingTile, Tile},
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
	//world: &World,
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
	mut q_falling_tile: Query<(Entity, &Tile, &FallingTile)>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	time: Res<Time>,
	mut timer: ResMut<TickTimer>,
) {
	if !timer.0.tick(time.delta()).just_finished() {
		return;
	}
	let mut tuples = vec![];
	for (entity, tile, falling_tile) in q_falling_tile.iter_mut() {
		tuples.push((entity, tile, falling_tile.0));
	}
	if tuples.len() == 0 {
		return;
	}
	tuples.sort_by(|a, b| a.2.cmp(&b.2));
	for tuple in tuples {
		let current_position = tuple.1.coordinate.as_tile_coord();
		let down = current_position.moved(&Vec2::NEG_Y);
		match map.get_tile(down) {
			Ok(opt) => match opt {
				Some(_) => {
					let _ = &commands.entity(tuple.0).remove::<FallingTile>();
					//slide
				}
				None => {
					let e =
						create_tile_entity(&mut commands, &asset_server, down, tuple.1.tile_type);
					if let Err(_) = map.set_tile(&mut commands, current_position, None) {
						//unloaded chunk
					};
					commands.entity(e).insert(FallingTile(down.y_i32()));
					if let Err(_) = map.set_tile(&mut commands, down, Some(e)) {
						//unloaded chunk
					};
				}
			},
			Err(_) => {
				let _ = &commands.entity(tuple.0).remove::<FallingTile>(); //unloaded chunk
				continue;
			}
		};
	}
}

pub struct UpdateTilePhysicsEvent(pub Coordinate);
