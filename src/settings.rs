use crate::inputs::KeyBinds;
use bevy::prelude::Resource;

#[derive(Resource)]
pub struct Settings {
	pub hold_to_keep_jumping: bool,
	pub keybinds: KeyBinds,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			hold_to_keep_jumping: false,
			keybinds: Default::default(),
		}
	}
}
