use crate::inputs::KeyBinds;
use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct Settings {
	pub hold_to_keep_jumping: bool,
	pub keybinds: KeyBinds,
}
