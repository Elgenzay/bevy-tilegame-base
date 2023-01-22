use crate::{Velocity, AIR_CONTROL, AIR_FRICTION, PLAYER_ACCEL, PLAYER_JUMP_FORCE, PLAYER_SPEED};
use bevy::{
	prelude::{App, Component, Input, KeyCode, Plugin, Query, Res},
	time::Time,
};

pub struct Players;

impl Plugin for Players {
	fn build(&self, app: &mut App) {
		app.add_system(move_player);
	}
}

#[derive(Component)]
pub struct Player {
	pub on_ground: bool,
	pub look_direction: LookDirection,
}

pub enum LookDirection {
	Left,
	Right,
}

fn move_player(
	keyboard_input: Res<Input<KeyCode>>,
	mut query: Query<(&mut Player, &mut Velocity)>,
	time: Res<Time>,
) {
	let mut direction = 0.0;
	if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
		direction -= 1.0;
	}
	if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
		direction += 1.0;
	}
	let (mut player, mut velocity) = query.single_mut();
	let mut jumping = false;
	if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
		jumping = true;
	};
	if player.on_ground {
		if jumping {
			player.on_ground = false;
			velocity.y = PLAYER_JUMP_FORCE;
		}
		if direction == 0.0 {
			velocity.x = 0.0;
			return;
		}
		velocity.x += direction * PLAYER_ACCEL * time.delta_seconds();
	} else {
		if direction != 0.0 {
			velocity.x += direction * PLAYER_ACCEL * AIR_CONTROL * time.delta_seconds();
		}
		velocity.x = velocity.x.clamp(-PLAYER_SPEED, PLAYER_SPEED);
		if velocity.x > 0.0 {
			velocity.x -= AIR_FRICTION * time.delta_seconds();
		} else if velocity.x < 0.0 {
			velocity.x += AIR_FRICTION * time.delta_seconds();
		}
	}
	velocity.x = velocity.x.clamp(-PLAYER_SPEED, PLAYER_SPEED);
}
