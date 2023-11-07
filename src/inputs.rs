use crate::{
	players::{Jumping, MoveDirection, Player},
	settings::Settings,
	ScreenCursor, WorldCursor,
};
use bevy::{
	prelude::{
		App, Camera, GlobalTransform, Input, KeyCode, Plugin, Query, Res, Transform, Update, With,
		Without,
	},
	ui::{Style, Val},
	window::Window,
};

pub struct Inputs;

impl Plugin for Inputs {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, mouse_events_system)
			.add_systems(Update, keyboard_events_system);
	}
}

pub struct KeyBinds {
	pub move_left: KeyBind,
	pub move_right: KeyBind,
	pub jump: KeyBind,
}

impl Default for KeyBinds {
	fn default() -> Self {
		Self {
			move_left: KeyBind {
				primary: Some(KeyCode::A),
				secondary: Some(KeyCode::Left),
			},
			move_right: KeyBind {
				primary: Some(KeyCode::D),
				secondary: Some(KeyCode::Right),
			},
			jump: KeyBind {
				primary: Some(KeyCode::W),
				secondary: Some(KeyCode::Space),
			},
		}
	}
}

pub struct KeyBind {
	pub primary: Option<KeyCode>,
	pub secondary: Option<KeyCode>,
}

impl KeyBind {
	pub fn is_pressed(&self, input: &Input<KeyCode>) -> bool {
		if let Some(v) = self.primary {
			if input.pressed(v) {
				return true;
			}
		}
		if let Some(v) = self.secondary {
			if input.pressed(v) {
				return true;
			}
		}
		false
	}

	pub fn just_pressed(&self, input: &Input<KeyCode>) -> bool {
		if let Some(v) = self.primary {
			if input.just_pressed(v) {
				return true;
			}
		}
		if let Some(v) = self.secondary {
			if input.just_pressed(v) {
				return true;
			}
		}
		false
	}
}

fn keyboard_events_system(
	input: Res<Input<KeyCode>>,
	settings: Res<Settings>,
	mut q_player: Query<(&Player, &mut MoveDirection, &mut Jumping)>,
) {
	for (player, mut move_direction, mut jumping) in &mut q_player {
		if let Player::Local = player {
			let mut dir = MoveDirection::None;
			if settings.keybinds.move_left.is_pressed(&input) {
				dir = MoveDirection::Left;
			}
			if settings.keybinds.move_right.is_pressed(&input) {
				if let MoveDirection::Left = dir {
					dir = MoveDirection::None;
				} else {
					dir = MoveDirection::Right;
				}
			}
			*move_direction = dir;
			if settings.hold_to_keep_jumping {
				jumping.0 = settings.keybinds.jump.is_pressed(&input);
			} else {
				jumping.0 = settings.keybinds.jump.just_pressed(&input);
			}
			return;
		}
	}
}

fn mouse_events_system(
	wnds: Query<&Window>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut q_worldcursor: Query<&mut Transform, With<WorldCursor>>,
	mut q_screencursor: Query<&mut Style, (With<ScreenCursor>, Without<WorldCursor>)>,
	//input: Res<Input<MouseButton>>,
) {
	let (camera, camera_transform) = match q_camera.get_single() {
		Ok(v) => v,
		Err(_) => return,
	};
	let wnd = if let Ok(v) = wnds.get_single() {
		v
	} else {
		return;
	};

	if let Some(screen_pos) = wnd.cursor_position() {
		camera
			.viewport_to_world(camera_transform, screen_pos)
			.map(|ray| ray.origin.truncate())
			.map(|world_position| {
				if let Ok(mut screencursor) = q_screencursor.get_single_mut() {
					screencursor.left = Val::Px(screen_pos.x);
					screencursor.top = Val::Px(screen_pos.y);
				}

				if let Ok(mut worldcursor) = q_worldcursor.get_single_mut() {
					if worldcursor.translation.truncate() != world_position {
						worldcursor.translation = world_position.extend(worldcursor.translation.z);
					}
				}
			});
	}
}
