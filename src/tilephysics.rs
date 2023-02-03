use bevy::prelude::{App, Plugin};

use crate::grid::{Coordinate, Map};

pub struct TilePhysics;

impl Plugin for TilePhysics {
	fn build(&self, app: &mut App) {
		app.add_system(apply_gravity);
	}
}

pub fn update_tile_physics(mut map: Map, coordinate: Coordinate) {
	let tiles = match coordinate {
		Coordinate::Chunk { x: _, y: _ } => match map.get_tiles(coordinate) {
			Ok(v) => v,
			Err(_) => return,
		},
		Coordinate::ChunkLocal { x: _, y: _ } => {
			panic!("ChunkLocal coordinate passed toupdate_tile_physics()")
		}
		Coordinate::Tile { x: _, y: _ } | Coordinate::World { x: _, y: _ } => {
			match map.get_tile(coordinate) {
				Ok(opt) => match opt {
					Some(tile) => vec![tile],
					None => return,
				},
				Err(_) => return,
			}
		}
	};
	for tile in tiles {
		//
	}
}

fn apply_gravity() {}
