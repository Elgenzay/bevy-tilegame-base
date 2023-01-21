use bevy::{
	prelude::{
		App, Children, Component, Deref, DerefMut, Plugin, Query, Res, Transform, Vec2, Vec3, With,
	},
	time::Time,
};

use crate::{
	grid::{region_collides, Chunk, Region},
	Player, PLAYER_SIZE, TERMINAL_VELOCITY,
};

pub struct Physics;

impl Plugin for Physics {
	fn build(&self, app: &mut App) {
		app.add_system(apply_velocity).add_system(apply_gravity);
	}
}

#[derive(Component)]
pub struct Collider;

#[derive(Component, Deref, DerefMut)]
pub struct Gravity(pub f32);

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

fn apply_velocity(
	time: Res<Time>,
	q_colliders: Query<&Region, With<Collider>>,
	q_chunks: Query<(&Region, &Children), With<Chunk>>,
	mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
	for (mut player_transform, mut player_velocity) in &mut player_query {
		if player_velocity.x == 0.0 && player_velocity.y == 0.0 {
			continue;
		}
		let delta_y = player_velocity.y * time.delta_seconds();
		let new_pos = Vec3::new(
			player_transform.translation.x + (player_velocity.x * time.delta_seconds()),
			player_transform.translation.y + delta_y,
			0.0,
		);
		let new_player_region = Region::from_size(
			&Vec2::new(
				new_pos.x - (PLAYER_SIZE.x as f32 * 0.5),
				new_pos.y - (PLAYER_SIZE.y as f32 * 0.5),
			),
			&PLAYER_SIZE.as_vec2(),
		);
		if !region_collides(&new_player_region, &q_colliders, &q_chunks) {
			player_transform.translation = new_pos;
		} else if player_velocity.x != 0.0 {
			player_velocity.x = 0.0;
			let new_player_region = Region::from_size(
				&Vec2::new(
					player_transform.translation.x - (PLAYER_SIZE.x as f32 * 0.5),
					player_transform.translation.y - (PLAYER_SIZE.y as f32 * 0.5),
				),
				&PLAYER_SIZE.as_vec2(),
			)
			.moved(&Vec2::new(0.0, delta_y));
			if !region_collides(&new_player_region, &q_colliders, &q_chunks) {
				player_transform.translation.y += delta_y;
			} else {
				player_velocity.y = 0.0;
			}
		} else {
			player_velocity.y = 0.0;
		}
	}
}

fn apply_gravity(
	mut query: Query<(&Gravity, &mut Velocity, &Transform, &mut Player)>,
	time: Res<Time>,
	q_colliders: Query<&Region, With<Collider>>,
	q_chunks: Query<(&Region, &Children), With<Chunk>>,
) {
	for (gravity, mut velocity, transform, mut player) in &mut query {
		let floor_check = Region::from_size(
			&Vec2::new(
				transform.translation.x - (PLAYER_SIZE.x as f32 * 0.5),
				transform.translation.y - (PLAYER_SIZE.y as f32 * 0.5),
			),
			&PLAYER_SIZE.as_vec2(),
		)
		.moved(&Vec2::new(0.0, -1.0));
		if region_collides(&floor_check, &q_colliders, &q_chunks) {
			player.on_ground = true;
			if velocity.y < 0.0 {
				velocity.y = 0.0;
			}
			continue;
		}
		player.on_ground = false;
		if velocity.y < -TERMINAL_VELOCITY {
			continue;
		}
		velocity.y = velocity.y - (gravity.0 * time.delta_seconds());
	}
}
