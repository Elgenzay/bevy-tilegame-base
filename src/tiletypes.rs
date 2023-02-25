use crate::FLUID_PER_TILE;

#[derive(Copy, Clone, PartialEq)]
pub enum TileType {
	Empty,
	Gravel,
	Moss,
	Dirt,
	Sand,
	Water(u8),
}

pub enum MatterState {
	Solid,
	Liquid,
}

impl TileType {
	pub fn all() -> Vec<TileType> {
		vec![
			TileType::Empty,
			TileType::Gravel,
			TileType::Moss,
			TileType::Dirt,
			TileType::Sand,
			TileType::Water(0),
		]
	}

	pub fn get_name(&self) -> String {
		match self {
			TileType::Empty => "Empty",
			TileType::Gravel => "Gravel",
			TileType::Moss => "Moss",
			TileType::Dirt => "Dirt",
			TileType::Sand => "Sand",
			TileType::Water(_) => "Water",
		}
		.to_owned()
	}

	pub fn get_sprite_dir_name(&self) -> String {
		match self {
			TileType::Gravel => "gravel",
			TileType::Moss => "moss",
			TileType::Dirt => "dirt",
			TileType::Sand => "sand",
			TileType::Water(_) => "water",
			_ => panic!("Tried to get_sprite_dir_name() of an invisible tiletype"),
		}
		.to_owned()
	}

	pub fn morph_sprite(&self) -> bool {
		match self {
			_ => true,
		}
	}

	pub fn is_weighted(&self) -> bool {
		if self.is_liquid() {
			return true;
		}
		match self {
			TileType::Gravel => true,
			TileType::Sand => true,
			_ => false,
		}
	}

	pub fn get_matter_state(&self) -> Option<MatterState> {
		if let TileType::Empty = self {
			None
		} else {
			Some(match self {
				TileType::Water(_) => MatterState::Liquid,
				_ => MatterState::Solid,
			})
		}
	}

	pub fn get_granularity(&self) -> u8 {
		if self.is_liquid() {
			return (FLUID_PER_TILE - self.get_liquid_level()) + 1;
		}
		match self {
			TileType::Gravel => 1,
			TileType::Sand => 2,
			_ => 0,
		}
	}

	pub fn get_liquid_level(&self) -> u8 {
		match self {
			TileType::Water(l) => *l,
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
		if let Some(MatterState::Solid) = self.get_matter_state() {
			true
		} else {
			false
		}
	}

	pub fn is_liquid(&self) -> bool {
		if let Some(MatterState::Liquid) = self.get_matter_state() {
			true
		} else {
			false
		}
	}

	pub fn with_liquid_level(&self, level: u8) -> TileType {
		match self {
			TileType::Water(_) => TileType::Water(level),
			_ => panic!("with_liquid_level() not implemented for passed tiletype"),
		}
	}

	pub fn is_obstructed_by(&self, other: TileType) -> bool {
		if other.is_solid() {
			return true;
		}
		if self.is_liquid() && other.is_liquid() {
			return true;
		}
		false
	}
}
