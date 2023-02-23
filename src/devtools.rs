use bevy::{
	input::mouse::MouseWheel,
	prelude::{
		App, BuildChildren, Color, Commands, Component, Entity, EventReader, EventWriter, Input,
		KeyCode, MouseButton, OrthographicProjection, Plugin, Query, Res, ResMut, Resource,
		StartupStage, TextBundle, Transform, Vec2, With,
	},
	text::{Text, TextAlignment, TextStyle},
	time::Time,
	ui::{PositionType, Style, UiRect, Val},
};

use crate::{
	grid::{Coordinate, CreateTileEvent, DestroyTileEvent, Map},
	playerphysics::Position,
	players::Player,
	sprites::Sprites,
	tiletypes::TileType,
	MainCamera, UIWrapper, WorldCursor, CAMERA_PROJECTION_SCALE,
};

pub struct DevTools;

impl Plugin for DevTools {
	fn build(&self, app: &mut App) {
		app.add_system(place_tiles)
			.add_system(camera_zoom)
			.add_system(update_info)
			.add_startup_system_to_stage(StartupStage::PostStartup, setup_devtools);
	}
}

#[derive(Component)]
struct DebugInfo;

fn place_tiles(
	q_cursor: Query<&Transform, With<WorldCursor>>,
	mut ev_destroytile: EventWriter<DestroyTileEvent>,
	mut ev_createtile: EventWriter<CreateTileEvent>,
	kb_input: Res<Input<KeyCode>>,
	m_input: Res<Input<MouseButton>>,
	mut q_camera: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
	if let Ok(cursor_pos) = q_cursor.get_single() {
		let world_coord = Coordinate::world_coord_from_vec2(cursor_pos.translation.truncate());

		if kb_input.pressed(KeyCode::Key1) {
			ev_createtile.send(CreateTileEvent(world_coord, TileType::Sand));
		} else if kb_input.pressed(KeyCode::Key2) {
			ev_createtile.send(CreateTileEvent(world_coord, TileType::Dirt));
		} else if kb_input.pressed(KeyCode::Key3) {
			ev_createtile.send(CreateTileEvent(world_coord, TileType::Gravel));
		} else if kb_input.pressed(KeyCode::Key4) {
			ev_createtile.send(CreateTileEvent(world_coord, TileType::Moss));
		}
		let size = if m_input.pressed(MouseButton::Left) {
			1
		} else if m_input.pressed(MouseButton::Right) {
			10
		} else {
			0
		};
		if size != 0 {
			for x in -size..=size {
				for y in -size..=size {
					ev_destroytile.send(DestroyTileEvent(
						world_coord
							.as_tile_coord()
							.moved(&Vec2::new(x as f32, y as f32)),
					));
				}
			}
		}
		if m_input.just_pressed(MouseButton::Middle) {
			let mut camera_projection = q_camera.single_mut();
			camera_projection.scale = CAMERA_PROJECTION_SCALE;
		}
	}
}

fn camera_zoom(
	mut mouse_wheel_events: EventReader<MouseWheel>,
	mut q_camera: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
	for event in mouse_wheel_events.iter() {
		if let Ok(mut camera_projection) = q_camera.get_single_mut() {
			camera_projection.scale += {
				if event.y > 0.0 {
					-0.05
				} else if event.y < 0.0 {
					0.05
				} else {
					return;
				}
			};
			if camera_projection.scale < 0.1 {
				camera_projection.scale = 0.1;
			}
		};
	}
}

fn setup_devtools(mut commands: Commands, q_wrapper: Query<Entity, With<UIWrapper>>) {
	commands.insert_resource(FrameRate {
		frame_times: vec![],
		avg_frame_rate: 0.0,
	});
	let info = commands
		.spawn((
			TextBundle::from_section(
				"",
				TextStyle {
					..Default::default()
				},
			)
			.with_style(Style {
				position_type: PositionType::Absolute,
				position: UiRect {
					top: Val::Px(10.0),
					left: Val::Px(10.0),
					..Default::default()
				},
				..Default::default()
			})
			.with_text_alignment(TextAlignment::TOP_LEFT),
			DebugInfo,
		))
		.id();

	let w = q_wrapper.single();
	commands.entity(w).add_child(info);
}

#[derive(Resource)]
struct FrameRate {
	frame_times: Vec<f32>,
	avg_frame_rate: f32,
}

fn update_info(
	mut q_info: Query<&mut Text, With<DebugInfo>>,
	sprites: Res<Sprites>,
	q_player: Query<(&Player, &Position)>,
	q_cursor: Query<&Transform, With<WorldCursor>>,
	map: Res<Map>,
	time: Res<Time>,
	mut framerate: ResMut<FrameRate>,
) {
	let fps = 1.0 / time.delta_seconds();
	framerate.frame_times.push(fps);

	if framerate.frame_times.len() >= 60 {
		let total_time: f32 = framerate.frame_times.iter().sum();
		framerate.avg_frame_rate = total_time / 60.0;
		framerate.frame_times.remove(0);
	}

	if let Ok(mut t) = q_info.get_single_mut() {
		let player_pos = {
			let mut opt = None;
			for p in q_player.iter() {
				if let Player::Local = p.0 {
					opt = Some(p.1);
				}
			}
			if let Some(t) = opt {
				t
			} else {
				panic!()
			}
		}
		.0;
		let cursor_pos = if let Ok(v) = q_cursor.get_single() {
			v.translation.truncate()
		} else {
			Vec2::ZERO
		};
		let cursor_pos = Coordinate::world_coord_from_vec2(cursor_pos);
		let player_pos = Coordinate::world_coord_from_vec2(player_pos);
		let (ct_name, ct_weighted, ct_granularity) = if let Ok(t) = map.get_tile(cursor_pos) {
			(
				t.tile_type.get_name(),
				t.tile_type.is_weighted().to_string(),
				t.tile_type.get_granularity().to_string(),
			)
		} else {
			("null".to_owned(), "null".to_owned(), "null".to_owned())
		};

		let info = format!(
			"
			FPS: {:.0}
			\n
			player pos   tile: ({},{})\n
			            world: ({},{})\n
			            chunk: ({},{})\n
			       chunklocal: ({},{})\n
			\n
			cursor pos   tile: ({},{})\n
			            world: ({},{})\n
			            chunk: ({},{})\n
			       chunklocal: ({},{})\n
			\n
			cursor tile  name: {}\n
			         weighted: {}\n
			      granularity: {}",
			framerate.avg_frame_rate,
			player_pos.as_tile_coord().x_i32(),
			player_pos.as_tile_coord().y_i32(),
			player_pos.x_i32(),
			player_pos.y_i32(),
			player_pos.as_chunk_coord().x_i32(),
			player_pos.as_chunk_coord().y_i32(),
			player_pos.as_chunklocal_coord().x_i32(),
			player_pos.as_chunklocal_coord().y_i32(),
			cursor_pos.as_tile_coord().x_i32(),
			cursor_pos.as_tile_coord().y_i32(),
			cursor_pos.x_i32(),
			cursor_pos.y_i32(),
			cursor_pos.as_chunk_coord().x_i32(),
			cursor_pos.as_chunk_coord().y_i32(),
			cursor_pos.as_chunklocal_coord().x_i32(),
			cursor_pos.as_chunklocal_coord().y_i32(),
			ct_name,
			ct_weighted,
			ct_granularity
		);

		*t = Text::from_section(
			info,
			TextStyle {
				font: sprites.fonts.get("pressstart2p").unwrap().clone(),
				font_size: 10.0,
				color: Color::WHITE,
			},
		)
	};
}
