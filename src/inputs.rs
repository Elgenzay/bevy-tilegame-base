use bevy::{
	prelude::{
		App, AssetServer, Camera, Commands, GlobalTransform, Input, KeyCode, MouseButton, Plugin,
		Query, Res, ResMut, Transform, Vec2, Vec3, With,
	},
	render::camera::RenderTarget,
	time::Time,
	window::Windows,
};

use crate::{
	grid::{Coordinate, Map},
	Cursor, Player, Velocity, AIR_CONTROL, AIR_FRICTION, GROUND_FRICTION, PLAYER_ACCEL,
	PLAYER_SPEED,
};

pub struct Inputs;

impl Plugin for Inputs {
	fn build(&self, app: &mut App) {
		app.add_system(mouse_events_system).add_system(move_player);
	}
}

fn mouse_events_system(
	wnds: Res<Windows>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut query: Query<&mut Transform, With<Cursor>>,
	mut map: ResMut<Map>,
	input: Res<Input<MouseButton>>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	let (camera, camera_transform) = q_camera.single();
	let wnd = if let RenderTarget::Window(id) = camera.target {
		wnds.get(id).unwrap()
	} else {
		wnds.get_primary().unwrap()
	};
	if let Some(screen_pos) = wnd.cursor_position() {
		let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);
		let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
		let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();
		let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
		let world_pos: Vec2 = world_pos.truncate();
		let cursorlocation = Vec3::new(world_pos.x.floor(), world_pos.y.floor(), 1.0);
		if query.single_mut().translation != cursorlocation {
			query.single_mut().translation = cursorlocation;
			let world_coord = Coordinate::from_vec2(world_pos);
			let tile = map.get_tile(world_coord);
			if let Some(_) = tile {
				let t_coord = world_coord.as_tile();
				println!(
					"({},{})",
					t_coord.x_i32().to_string(),
					t_coord.y_i32().to_string()
				);
			}
		}
		if input.just_pressed(MouseButton::Left) {
			//todo
		}
	}
}

fn move_player(
	keyboard_input: Res<Input<KeyCode>>,
	mut query: Query<(&Player, &mut Velocity)>,
	time: Res<Time>,
) {
	let mut direction = 0.0;
	if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
		direction -= 1.0;
	}
	if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
		direction += 1.0;
	}
	let (player, mut velocity) = query.single_mut();
	let mut jumping = false;
	if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
		jumping = true;
	};
	if player.on_ground {
		if jumping {
			velocity.y = super::PLAYER_JUMP_FORCE;
		}
		if direction == 0.0 {
			velocity.x = 0.0;
			return;
		}
		if velocity.x > PLAYER_SPEED {
			velocity.x -= GROUND_FRICTION * time.delta_seconds();
			return;
		}
		if velocity.x < -PLAYER_SPEED {
			velocity.x += GROUND_FRICTION * time.delta_seconds();
			return;
		}
		velocity.x += direction * PLAYER_ACCEL * time.delta_seconds();
		return;
	}
	if velocity.x > 0.0 {
		velocity.x -= AIR_FRICTION * time.delta_seconds();
	} else if velocity.x < 0.0 {
		velocity.x += AIR_FRICTION * time.delta_seconds();
	}
	if direction != 0.0 {
		velocity.x += direction * PLAYER_ACCEL * AIR_CONTROL * time.delta_seconds();
	}
}
