use bevy::prelude::{
	App, AssetServer, BuildChildren, Commands, Entity, EventReader, Plugin, Query, Res, ResMut,
	Vec2, With, World,
};

use crate::grid::{create_tile_entity, Coordinate, FallingTile, Map, Tile, TileType, WeightedTile};

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
				commands.entity(*tile_parent).insert(FallingTile);
			}
		}
	}
}

fn apply_gravity(
	q_falling_tile: Query<(Entity, &Tile), With<FallingTile>>,
	//mut map: ResMut<Map>,
	mut commands: Commands,
	//asset_server: Res<AssetServer>,
) {
	for (falling_tile_entity, tile) in q_falling_tile.iter() {
		let _ = &commands.entity(falling_tile_entity).remove::<FallingTile>();
	}
}

pub struct UpdateTilePhysicsEvent(pub Coordinate);
