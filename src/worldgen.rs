use crate::tiletypes::TileType;
use noise::{NoiseFn, Simplex};

pub fn tiletype_at(x: i32, y: i32) -> TileType {
	let gen_x = x as f64 * 0.025;
	let gen_y = y as f64 * 0.025;

	let simplex = Simplex::new(1337);
	let noise = simplex.get([gen_x, gen_y]);

	if noise < 0.0 {
		return TileType::Empty;
	}

	if noise < 0.01 {
		TileType::Moss
	} else if noise > 0.2 {
		TileType::Gravel
	} else {
		TileType::Dirt
	}
}
