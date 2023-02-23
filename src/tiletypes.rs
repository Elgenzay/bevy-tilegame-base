#[derive(Copy, Clone, PartialEq)]
pub enum TileType {
	Empty,
	Gravel,
	Moss,
	Dirt,
	Sand,
}

impl TileType {
	pub fn all() -> Vec<TileType> {
		vec![
			TileType::Empty,
			TileType::Gravel,
			TileType::Moss,
			TileType::Dirt,
			TileType::Sand,
		]
	}

	pub fn get_name(&self) -> String {
		match self {
			TileType::Empty => "Empty",
			TileType::Gravel => "Gravel",
			TileType::Moss => "Moss",
			TileType::Dirt => "Dirt",
			TileType::Sand => "Sand",
		}
		.to_owned()
	}

	pub fn get_sprite_dir_name(&self) -> String {
		match self {
			TileType::Gravel => "gravel",
			TileType::Moss => "moss",
			TileType::Dirt => "dirt",
			TileType::Sand => "sand",
			_ => panic!("Tried to get_sprite_dir_name() of an invisible tiletype"),
		}
		.to_owned()
	}

	pub fn is_weighted(&self) -> bool {
		match self {
			TileType::Gravel => true,
			TileType::Sand => true,
			_ => false,
		}
	}

	pub fn is_liquid(&self) -> bool {
		match self {
			_ => false,
		}
	}

	pub fn get_granularity(&self) -> u8 {
		match self {
			TileType::Gravel => 1,
			TileType::Sand => 2,
			_ => 0,
		}
	}

	pub fn is_visible(&self) -> bool {
		match self {
			TileType::Empty => false,
			_ => true,
		}
	}

	pub fn is_solid(&self) -> bool {
		if self.is_liquid() || !self.is_visible() {
			false
		} else {
			true
		}
	}
}
