use bevy::{
	prelude::{
		App, Camera, EventWriter, GlobalTransform, Input, MouseButton, Plugin, Query, Res, ResMut,
		Transform, Vec2, Vec3, With,
	},
	render::camera::RenderTarget,
	window::Windows,
};

use crate::{
	grid::{Coordinate, DestroyTileEvent, Map},
	Cursor,
};

pub struct Inputs;

impl Plugin for Inputs {
	fn build(&self, app: &mut App) {
		app.add_system(mouse_events_system);
	}
}

fn mouse_events_system(
	wnds: Res<Windows>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut query: Query<&mut Transform, With<Cursor>>,
	map: ResMut<Map>,
	input: Res<Input<MouseButton>>,
	mut ev_destroytile: EventWriter<DestroyTileEvent>,
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
		let world_coord = Coordinate::from_vec2(world_pos);
		if query.single_mut().translation != cursorlocation {
			query.single_mut().translation = cursorlocation;
			let tile = map.get_tile(world_coord);
			if let Some(_) = tile {
				//let t_coord = world_coord.as_tile();
				//	println!(
				//		"({},{})",
				//		t_coord.x_i32().to_string(),
				//		t_coord.y_i32().to_string()
				//	);
			}
		}
		if input.just_pressed(MouseButton::Left) {
			//println!("click");
			ev_destroytile.send(DestroyTileEvent(world_coord));
		}
	}
}
