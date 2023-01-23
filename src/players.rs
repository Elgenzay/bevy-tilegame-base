use crate::{
	physics::{Gravity, Position},
	Cursor, MainCamera, Velocity, AIR_CONTROL, AIR_FRICTION, PLAYER_ACCEL, PLAYER_JUMP_FORCE,
	PLAYER_SPEED,
};
use bevy::{
	prelude::{App, Bundle, Component, Plugin, Query, Res, Transform, Vec2, Vec3, With, Without},
	time::Time,
};

pub struct Players;

impl Plugin for Players {
	fn build(&self, app: &mut App) {
		app.add_system(move_player).add_system(camera_follow);
	}
}

#[derive(Component)]
pub enum Player {
	Local,
	Remote,
}

impl Default for Player {
	fn default() -> Self {
		Self::Local
	}
}

#[derive(Component)]
pub enum LookDirection {
	Left,
	Right,
}

impl Default for LookDirection {
	fn default() -> Self {
		Self::Right
	}
}

#[derive(Component)]
pub enum MoveDirection {
	Left,
	Right,
	None,
}

impl Default for MoveDirection {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Component)]
pub struct Jumping(pub bool);

impl Default for Jumping {
	fn default() -> Self {
		Self(false)
	}
}

#[derive(Component)]
pub struct OnGround(pub bool);

impl Default for OnGround {
	fn default() -> Self {
		Self(false)
	}
}

#[derive(Bundle)]
pub struct PlayerBundle {
	pub player: Player,
	pub velocity: Velocity,
	pub gravity: Gravity,
	pub on_ground: OnGround,
	pub look_direction: LookDirection,
	pub move_direction: MoveDirection,
	pub jumping: Jumping,
	pub position: Position,
}

impl Default for PlayerBundle {
	fn default() -> Self {
		Self {
			player: Default::default(),
			velocity: Default::default(),
			gravity: Default::default(),
			on_ground: Default::default(),
			look_direction: Default::default(),
			move_direction: Default::default(),
			jumping: Default::default(),
			position: Default::default(),
		}
	}
}

fn move_player(
	mut q_player: Query<
		(
			&mut Velocity,
			&mut LookDirection,
			&mut OnGround,
			&MoveDirection,
			&Jumping,
		),
		With<Player>,
	>,
	time: Res<Time>,
) {
	for (mut velocity, mut look_direction, mut on_ground, move_direction, jumping) in &mut q_player
	{
		let direction = match move_direction {
			MoveDirection::Left => {
				*look_direction = LookDirection::Left;
				-1.0
			}
			MoveDirection::Right => {
				*look_direction = LookDirection::Right;
				1.0
			}
			MoveDirection::None => 0.0,
		};
		if on_ground.0 {
			if jumping.0 {
				on_ground.0 = false;
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
}

fn camera_follow(
	mut q_camera: Query<&mut Transform, With<MainCamera>>,
	q_player: Query<(&Player, &Position)>,
	q_cursor: Query<&Transform, (With<Cursor>, Without<MainCamera>)>,
) {
	let mut camera_transform = match q_camera.get_single_mut() {
		Ok(v) => v,
		Err(_) => return,
	};
	let cursor_transform = match q_cursor.get_single() {
		Ok(v) => v,
		Err(_) => return,
	};

	for (player, player_position) in q_player.into_iter() {
		if let Player::Local = player {
			let target = Vec3::new(
				player_position.0.x,
				player_position.0.y,
				camera_transform.translation.z,
			);
			camera_transform.translation = camera_transform
				.translation
				.lerp(target.lerp(cursor_transform.translation, 0.01), 0.02);
			return;
		}
	}
}
