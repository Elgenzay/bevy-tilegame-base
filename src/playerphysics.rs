use crate::{
	grid::{region_collides, Chunk, Region},
	players::OnGround,
	Player, GRAVITY_SCALE, PLAYER_SIZE, PLAYER_UNSTUCK_NUDGE_SPEED, TERMINAL_VELOCITY, TILE_SIZE,
};
use bevy::{
	prelude::{
		App, Children, Component, Deref, DerefMut, Plugin, Query, Res, Transform, Update, Vec2,
		With,
	},
	time::Time,
};

pub struct PlayerPhysics;

impl Plugin for PlayerPhysics {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, (apply_velocity, apply_gravity, motion_tween));
	}
}

#[derive(Component)]
pub struct Collider;

#[derive(Component, Deref, DerefMut)]
pub struct Gravity(pub f32);

impl Default for Gravity {
	fn default() -> Self {
		Self(GRAVITY_SCALE)
	}
}

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

impl Default for Velocity {
	fn default() -> Self {
		Self(Vec2::ZERO)
	}
}

#[derive(Component)]
pub struct Position(pub Vec2);

impl Default for Position {
	fn default() -> Self {
		Self(Vec2::ZERO)
	}
}

fn motion_tween(mut q_objects: Query<(&mut Transform, &Position)>) {
	for (mut transform, position) in &mut q_objects {
		if transform.translation.truncate() == position.0 {
			continue;
		}

		let pos_vec3 = position.0.extend(0.0);

		if transform.translation.distance(pos_vec3) < 1.0 {
			transform.translation = pos_vec3;
		}

		transform.translation = transform
			.translation
			.lerp(position.0.extend(transform.translation.z), 0.2);
	}
}

fn apply_velocity(
	time: Res<Time>,
	q_colliders: Query<&Region, With<Collider>>,
	q_chunks: Query<(&Region, &Children), With<Chunk>>,
	mut player_query: Query<(&mut Position, &mut Velocity, &mut OnGround), With<Player>>,
) {
	for (mut player_position, mut player_velocity, mut on_ground) in &mut player_query {
		let player_size_halved_x = PLAYER_SIZE.x as f32 * 0.5;
		let player_size_halved_y = PLAYER_SIZE.y as f32 * 0.5;

		let current_player_region = Region::from_size(
			&Vec2::new(
				player_position.0.x - player_size_halved_x,
				player_position.0.y - player_size_halved_y,
			),
			&PLAYER_SIZE.as_vec2(),
		);

		if region_collides(&current_player_region, &q_colliders, &q_chunks) {
			on_ground.0 = false;
			player_velocity.0 = Vec2::ZERO;
			'outer: for m in [0.5, 1.0, 1.5] {
				for t in [
					// nudge direction priority order
					(0, -1),  // bottom
					(-1, 0),  // left
					(1, 0),   // right
					(-1, -1), // bottom left
					(1, -1),  // bottom right
					(0, 1),   // top
					(-1, 1),  // top left
					(1, 1),   // top right
				] {
					if !region_collides(
						&current_player_region.moved(&Vec2::new(
							t.0 as f32 * TILE_SIZE.x as f32 * m,
							t.1 as f32 * TILE_SIZE.y as f32 * m,
						)),
						&q_colliders,
						&q_chunks,
					) {
						let dir = Vec2::new(
							t.0 as f32 * PLAYER_UNSTUCK_NUDGE_SPEED,
							t.1 as f32 * PLAYER_UNSTUCK_NUDGE_SPEED,
						);
						player_position.0 += dir * time.delta_seconds();
						break 'outer;
					}
				}
				//
				// player is buried
				//
			}
		}

		if player_velocity.x == 0.0 && player_velocity.y == 0.0 {
			continue;
		}

		let delta_x = player_velocity.x * time.delta_seconds();
		let delta_y = player_velocity.y * time.delta_seconds();
		let new_pos = Vec2::new(player_position.0.x + delta_x, player_position.0.y + delta_y);

		let new_player_region = Region::from_size(
			&Vec2::new(
				new_pos.x - player_size_halved_x,
				new_pos.y - player_size_halved_y,
			),
			&PLAYER_SIZE.as_vec2(),
		);

		if !region_collides(&new_player_region, &q_colliders, &q_chunks) {
			player_position.0 = new_pos;
		} else if !region_collides(
			&current_player_region.moved(&Vec2::new(delta_x, 0.0)),
			&q_colliders,
			&q_chunks,
		) {
			//head bonk. maintain x vel
			player_velocity.y = 0.0;
			player_position.0.x = new_pos.x;
		} else if player_velocity.x != 0.0 {
			let step_up_region = new_player_region.moved(&Vec2::new(0.0, TILE_SIZE.y as f32));

			if on_ground.0 && !region_collides(&step_up_region, &q_colliders, &q_chunks) {
				player_position.0 = Vec2::new(new_pos.x, new_pos.y + TILE_SIZE.y as f32);
			} else {
				player_velocity.x = 0.0;
				let new_player_region = current_player_region.moved(&Vec2::new(0.0, delta_y));

				if !region_collides(&new_player_region, &q_colliders, &q_chunks) {
					//stepping up
					player_position.0.y += delta_y;
				} else {
					player_velocity.y = 0.0;
				}
			}
		} else {
			player_velocity.y = 0.0;
		}
	}
}

fn apply_gravity(
	mut query: Query<(&Gravity, &mut Velocity, &mut Position, &mut OnGround), With<Player>>,
	time: Res<Time>,
	q_colliders: Query<&Region, With<Collider>>,
	q_chunks: Query<(&Region, &Children), With<Chunk>>,
) {
	for (gravity, mut velocity, mut position, mut on_ground) in &mut query {
		let player_size_halved_x = PLAYER_SIZE.x as f32 * 0.5;
		let player_size_halved_y = PLAYER_SIZE.y as f32 * 0.5;

		let current_player_region = Region::from_size(
			&Vec2::new(
				position.0.x - player_size_halved_x,
				position.0.y - player_size_halved_y,
			),
			&PLAYER_SIZE.as_vec2(),
		);

		if region_collides(&current_player_region, &q_colliders, &q_chunks) {
			on_ground.0 = false;
			continue;
		}

		let floor_check = Region::from_size(
			&Vec2::new(
				position.0.x - player_size_halved_x,
				position.0.y - player_size_halved_y,
			),
			&PLAYER_SIZE.as_vec2(),
		)
		.moved(&Vec2::new(0.0, -1.0));

		let new_on_ground = region_collides(&floor_check, &q_colliders, &q_chunks);

		if velocity.y <= 0.0 && new_on_ground {
			//standing
			on_ground.0 = true;

			if velocity.y < 0.0 {
				velocity.y = 0.0;
			}

			continue;
		}

		if !new_on_ground && on_ground.0 {
			//just began falling
			if region_collides(
				&floor_check.moved(&Vec2::new(0.0, -(TILE_SIZE.y as f32))),
				&q_colliders,
				&q_chunks,
			) {
				//stepping down
				on_ground.0 = false;
				position.0.y -= TILE_SIZE.y as f32;
				continue;
			}
		}

		//falling
		on_ground.0 = false;

		if velocity.y < -TERMINAL_VELOCITY {
			continue;
		}

		velocity.y -= gravity.0 * time.delta_seconds();
	}
}
