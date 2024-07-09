use crate::light::Emitter;
use strum_macros::EnumIter;

#[derive(Copy, Clone, PartialEq, EnumIter)]
pub enum TileType {
	Empty,
	Gravel,
	Moss,
	Dirt,
	Sand,
	Water(Liquid),
	Magma(Liquid),
	Oil(Liquid),
	Lantern(Emitter),
}

impl TileType {
	pub fn get_name(&self) -> String {
		match self {
			TileType::Empty => "Empty",
			TileType::Gravel => "Gravel",
			TileType::Moss => "Moss",
			TileType::Dirt => "Dirt",
			TileType::Sand => "Sand",
			TileType::Water(_) => "Water",
			TileType::Magma(_) => "Magma",
			TileType::Oil(_) => "Oil",
			TileType::Lantern(_) => "Lantern",
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
			TileType::Magma(_) => "magma",
			TileType::Oil(_) => "oil",
			TileType::Lantern(_) => "lantern",
			_ => panic!(
				"get_sprite_dir_name() not implemented for passed tiletype: {}",
				self.get_name()
			),
		}
		.to_owned()
	}

	pub fn morph_sprite(&self) -> bool {
		true
	}

	pub fn is_weighted(&self) -> bool {
		if self.is_liquid() {
			return true;
		}

		matches!(self, TileType::Gravel | TileType::Sand)
	}

	pub fn get_matter_state(&self) -> Option<MatterState> {
		if let TileType::Empty = self {
			None
		} else if self.get_liquid().is_ok() {
			Some(MatterState::Liquid)
		} else {
			Some(MatterState::Solid)
		}
	}

	pub fn get_granularity(&self) -> u8 {
		if let Ok(liquid) = self.get_liquid() {
			return (self.get_fluidity() as f32
				* ((u8::MAX as f32 - liquid.level as f32) / u8::MAX as f32)) as u8;
		}
		match self {
			TileType::Gravel => 1,
			TileType::Sand => 2,
			_ => 0,
		}
	}

	pub fn get_liquid(&self) -> Result<Liquid, ()> {
		match self {
			TileType::Water(l) => Ok(*l),
			TileType::Magma(l) => Ok(*l),
			TileType::Oil(l) => Ok(*l),
			_ => Err(()),
		}
	}

	pub fn with_liquid(&self, l: Liquid) -> TileType {
		match self {
			TileType::Water(_) => TileType::Water(l),
			TileType::Magma(_) => TileType::Magma(l),
			TileType::Oil(_) => TileType::Oil(l),
			_ => panic!(
				"with_liquid() not implemented for passed tiletype: {}",
				self.get_name()
			),
		}
	}

	pub fn get_fluidity(&self) -> u8 {
		match self {
			TileType::Water(_) => 10,
			TileType::Magma(_) => 1,
			TileType::Oil(_) => 5,
			_ => panic!(
				"get_fluidity() not implemented for passed tiletype: {}",
				self.get_name()
			),
		}
	}

	pub fn get_liquid_interaction_with(&self, other: TileType) -> LiquidInteraction {
		match self {
			TileType::Water(_) => match other {
				TileType::Magma(_) => return LiquidInteraction::Vaporized,
				TileType::Oil(_) => return LiquidInteraction::Sink,
				_ => (),
			},
			TileType::Magma(_) => match other {
				TileType::Water(_) => return LiquidInteraction::Vaporize,
				TileType::Oil(_) => return LiquidInteraction::Vaporize,
				_ => (),
			},
			TileType::Oil(_) => match other {
				TileType::Water(_) => return LiquidInteraction::Float,
				TileType::Magma(_) => return LiquidInteraction::Vaporized,
				_ => (),
			},
			_ => (),
		}
		panic!(
			"get_liquid_interaction_with() not implemented: {} -> {}",
			self.get_name(),
			other.get_name()
		);
	}

	pub fn get_emitter(&self) -> Result<Emitter, ()> {
		match self {
			TileType::Lantern(e) => Ok(*e),
			_ => Err(()),
		}
	}

	pub fn is_emitter(&self) -> bool {
		self.get_emitter().is_ok()
	}

	pub fn liquid(&self) -> Liquid {
		if let Ok(l) = self.get_liquid() {
			l
		} else {
			panic!(
				"liquid() not implemented for passed tiletype: {}",
				self.get_name()
			)
		}
	}

	pub fn is_visible(&self) -> bool {
		!matches!(self, TileType::Empty)
	}

	pub fn is_solid(&self) -> bool {
		matches!(self.get_matter_state(), Some(MatterState::Solid))
	}

	pub fn is_liquid(&self) -> bool {
		matches!(self.get_matter_state(), Some(MatterState::Liquid))
	}

	pub fn is_opaque(&self) -> bool {
		!matches!(self, TileType::Water(_) | TileType::Empty)
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

pub enum MatterState {
	Solid,
	Liquid,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Liquid {
	pub level: u8,
	pub flowing_right: Option<bool>,
	pub momentum: u8,
	pub sprite_override: bool,
}

impl Default for Liquid {
	fn default() -> Self {
		Liquid {
			level: u8::MAX,
			flowing_right: None,
			momentum: 0,
			sprite_override: false,
		}
	}
}

pub enum LiquidInteraction {
	Vaporize,
	Vaporized,
	Float,
	Sink,
}
