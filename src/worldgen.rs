use noise::{NoiseFn, Simplex};

use crate::tiles::TileType;

pub fn tiletype_at(x: i32, y: i32) -> Option<TileType> {
	let gen_x = x as f64 * 0.025;
	let gen_y = y as f64 * 0.025;

	let simplex = Simplex::new(1337);
	let noise = simplex.get([gen_x, gen_y]);
	if noise < 0.0 {
		return None;
	}
	if noise < 0.01 {
		Some(TileType::Moss)
	} else if noise > 0.2 {
		Some(TileType::Gravel)
	} else {
		Some(TileType::Dirt)
	}
}
