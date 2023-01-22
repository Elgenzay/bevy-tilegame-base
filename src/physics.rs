use bevy::{
	prelude::{
		App, Children, Component, Deref, DerefMut, Plugin, Query, Res, Transform, Vec2, Vec3, With,
	},
	time::Time,
};

use crate::{
	grid::{region_collides, Chunk, Region},
	Player, PLAYER_SIZE, TERMINAL_VELOCITY, TILE_SIZE,
};

pub struct Physics;

impl Plugin for Physics {
	fn build(&self, app: &mut App) {
		app.add_system(apply_velocity)
			.add_system(apply_gravity)
			.add_system(motion_tween);
	}
}

#[derive(Component)]
pub struct Collider;

#[derive(Component, Deref, DerefMut)]
pub struct Gravity(pub f32);

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Position(pub Vec3);

fn motion_tween(mut q_objects: Query<(&mut Transform, &Position)>) {
	for (mut transform, position) in &mut q_objects {
		if transform.translation == position.0 {
			continue;
		}
		if transform.translation.distance(position.0) < 1.0 {
			transform.translation = position.0;
		}
		transform.translation = Vec3::new(
			transform.translation.x + ((position.0.x - transform.translation.x) * 0.1),
			transform.translation.y + ((position.0.y - transform.translation.y) * 0.1),
			0.0,
		);
	}
}

fn apply_velocity(
	time: Res<Time>,
	q_colliders: Query<&Region, With<Collider>>,
	q_chunks: Query<(&Region, &Children), With<Chunk>>,
	mut player_query: Query<(&mut Position, &mut Velocity, &Player)>,
) {
	for (mut player_position, mut player_velocity, player) in &mut player_query {
		if player_velocity.x == 0.0 && player_velocity.y == 0.0 {
			continue;
		}
		let delta_y = player_velocity.y * time.delta_seconds();
		let new_pos = Vec3::new(
			player_position.0.x + (player_velocity.x * time.delta_seconds()),
			player_position.0.y + delta_y,
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
			player_position.0 = new_pos;
		} else if player_velocity.x != 0.0 {
			let step_up_region = new_player_region.moved(&Vec2::new(0.0, TILE_SIZE.y));
			if player.on_ground && !region_collides(&step_up_region, &q_colliders, &q_chunks) {
				player_position.0 = Vec3::new(new_pos.x, new_pos.y + TILE_SIZE.y, 0.0);
			} else {
				player_velocity.x = 0.0;
				let new_player_region = Region::from_size(
					&Vec2::new(
						player_position.0.x - (PLAYER_SIZE.x as f32 * 0.5),
						player_position.0.y - (PLAYER_SIZE.y as f32 * 0.5),
					),
					&PLAYER_SIZE.as_vec2(),
				)
				.moved(&Vec2::new(0.0, delta_y));
				if !region_collides(&new_player_region, &q_colliders, &q_chunks) {
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
	mut query: Query<(&Gravity, &mut Velocity, &mut Position, &mut Player)>,
	time: Res<Time>,
	q_colliders: Query<&Region, With<Collider>>,
	q_chunks: Query<(&Region, &Children), With<Chunk>>,
) {
	for (gravity, mut velocity, mut position, mut player) in &mut query {
		let floor_check = Region::from_size(
			&Vec2::new(
				position.0.x - (PLAYER_SIZE.x as f32 * 0.5),
				position.0.y - (PLAYER_SIZE.y as f32 * 0.5),
			),
			&PLAYER_SIZE.as_vec2(),
		)
		.moved(&Vec2::new(0.0, -1.0));
		let new_on_ground = region_collides(&floor_check, &q_colliders, &q_chunks);
		if velocity.y <= 0.0 && new_on_ground {
			player.on_ground = true;
			if velocity.y < 0.0 {
				velocity.y = 0.0;
			}
			continue;
		}
		if !new_on_ground && player.on_ground {
			if region_collides(
				&floor_check.moved(&Vec2::new(0.0, -TILE_SIZE.y)),
				&q_colliders,
				&q_chunks,
			) {
				player.on_ground = false;
				position.0.y -= TILE_SIZE.y;
				continue;
			}
		}
		player.on_ground = false;
		if velocity.y < -TERMINAL_VELOCITY {
			continue;
		}
		velocity.y = velocity.y - (gravity.0 * time.delta_seconds());
	}
}
