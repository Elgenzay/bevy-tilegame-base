use bevy::{
	input::mouse::MouseWheel,
	prelude::{
		App, BuildChildren, Color, Commands, Component, CoreStage, Entity, EventReader,
		EventWriter, Input, KeyCode, MouseButton, OrthographicProjection, Plugin, Query, Res,
		ResMut, Resource, StartupStage, TextBundle, Transform, Vec2, Vec3, With,
	},
	sprite::SpriteBundle,
	text::{Text, TextAlignment, TextStyle},
	time::Time,
	ui::{PositionType, Style, UiRect, Val},
};

use crate::{
	grid::{Coordinate, CreateTileEvent, DestroyTileEvent, Map},
	playerphysics::Position,
	players::Player,
	sprites::Sprites,
	tilephysics::UpdateTileEvent,
	tiletypes::{Liquid, TileType},
	MainCamera, UIWrapper, WorldCursor, CAMERA_PROJECTION_SCALE,
};

pub struct DevTools;

impl Plugin for DevTools {
	fn build(&self, app: &mut App) {
		app.add_system(place_tiles)
			.add_event::<ToggleDebugUI>()
			.add_system_to_stage(CoreStage::First, debug_input)
			.add_system(camera_zoom)
			.add_system(tileupdate)
			.add_system(tickmarkers)
			.add_system(toggle_debug_ui_event)
			.add_system_to_stage(CoreStage::PostUpdate, update_info)
			.add_startup_system_to_stage(StartupStage::PostStartup, setup_devtools);
	}
}

#[derive(Component)]
struct DebugInfo;

#[derive(Resource)]
struct DebugStates {
	debug_ui_enabled: bool,
}

fn debug_input(kb_input: Res<Input<KeyCode>>, mut ev_toggleui: EventWriter<ToggleDebugUI>) {
	if kb_input.just_pressed(KeyCode::F1) {
		ev_toggleui.send(ToggleDebugUI());
	}
}

fn toggle_debug_ui_event(
	mut q_info: Query<&mut Text, With<DebugInfo>>,
	mut ev: EventReader<ToggleDebugUI>,
	mut states: ResMut<DebugStates>,
	sprites: Res<Sprites>,
) {
	for _ in ev.iter() {
		states.debug_ui_enabled = !states.debug_ui_enabled;
		if !states.debug_ui_enabled {
			if let Ok(mut t) = q_info.get_single_mut() {
				*t = Text::from_section(
					"",
					TextStyle {
						font: sprites.fonts.get("pressstart2p").unwrap().clone(),
						font_size: 10.0,
						color: Color::WHITE,
					},
				);
			}
		}
	}
}

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
		} else if kb_input.pressed(KeyCode::Key5) {
			ev_createtile.send(CreateTileEvent(
				world_coord,
				TileType::Water(Liquid::default()),
			));
		} else if kb_input.pressed(KeyCode::Key6) {
			ev_createtile.send(CreateTileEvent(
				world_coord,
				TileType::Magma(Liquid::default()),
			));
		} else if kb_input.pressed(KeyCode::Key7) {
			ev_createtile.send(CreateTileEvent(
				world_coord,
				TileType::Oil(Liquid::default()),
			));
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
	commands.insert_resource(DebugStates {
		debug_ui_enabled: false,
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
	state: Res<DebugStates>,
) {
	if !state.debug_ui_enabled {
		return;
	}
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
		let (ct_name, ct_weighted, ct_granularity, ct_liquidlevel, ct_momentum, ct_flowdir) =
			if let Ok(t) = map.get_tile(cursor_pos) {
				(
					t.tile_type.get_name(),
					t.tile_type.is_weighted().to_string(),
					t.tile_type.get_granularity().to_string(),
					if let Ok(liquid) = t.tile_type.get_liquid() {
						liquid.level.to_string()
					} else {
						"null".to_owned()
					},
					if let Ok(liquid) = t.tile_type.get_liquid() {
						liquid.momentum.to_string()
					} else {
						"null".to_owned()
					},
					if let Ok(liquid) = t.tile_type.get_liquid() {
						if let Some(v) = liquid.flowing_right {
							if v {
								"right".to_owned()
							} else {
								"left".to_owned()
							}
						} else {
							"none".to_owned()
						}
					} else {
						"null".to_owned()
					},
				)
			} else {
				(
					"null".to_owned(),
					"null".to_owned(),
					"null".to_owned(),
					"null".to_owned(),
					"null".to_owned(),
					"null".to_owned(),
				)
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
			      granularity: {}\n
			     liquid level: {}\n
			         momentum: {}\n
			          flowdir: {}",
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
			ct_granularity,
			ct_liquidlevel,
			ct_momentum,
			ct_flowdir
		);

		*t = Text::from_section(
			info,
			TextStyle {
				font: sprites.fonts.get("pressstart2p").unwrap().clone(),
				font_size: 10.0,
				color: Color::WHITE,
			},
		);
	};
}

fn tileupdate(
	mut commands: Commands,
	mut ev_update: EventReader<UpdateTileEvent>,
	sprites: Res<Sprites>,
	state: Res<DebugStates>,
) {
	if !state.debug_ui_enabled {
		return;
	}
	for ev in ev_update.iter() {
		let world_coord = ev.0.as_world_coord();
		commands.spawn((
			SpriteBundle {
				texture: sprites.debugtilemarker.clone(),
				transform: Transform::from_translation(Vec3::new(
					world_coord.x_f32(),
					world_coord.y_f32(),
					0.0,
				)),
				..Default::default()
			},
			UpdateMarker(3),
		));
	}
}

fn tickmarkers(mut commands: Commands, mut q_markers: Query<(Entity, &mut UpdateMarker)>) {
	for (entity, mut updatemarker) in q_markers.iter_mut() {
		updatemarker.0 -= 1;
		if updatemarker.0 == 0 {
			commands.entity(entity).despawn();
		}
	}
}

#[derive(Component)]
struct UpdateMarker(u8);

struct ToggleDebugUI();
