mod devtools;
mod grid;
mod inputs;
mod playerphysics;
mod players;
mod settings;
mod sprites;
mod tileoutline;
mod tilephysics;
mod tiles;
mod tiletypes;
mod worldgen;

use bevy::math::Vec3;
use bevy::prelude::*;
use devtools::DevTools;
use grid::Grid;
use inputs::Inputs;
use playerphysics::{PlayerPhysics, Velocity};
use players::{Player, PlayerBundle, Players};
use settings::Settings;
use sprites::{Sprites, SpritesPlugin};
use tilephysics::TilePhysics;

const WINDOW_DEFAULT_WIDTH: f32 = 1280.0;
const WINDOW_DEFAULT_HEIGHT: f32 = 720.0;

const CHUNK_SIZE: (u8, u8) = (32, 32);
const TILE_SIZE: UVec2 = UVec2::new(8, 8);
const RENDER_DISTANCE: UVec2 = UVec2::new(3, 2);
const UNRENDER_DISTANCE: UVec2 = UVec2::new(4, 3);
const CAMERA_PROJECTION_SCALE: f32 = 0.4;

const PLAYER_SIZE: UVec2 = UVec2::new(20, 36);
const PLAYER_ACCEL: f32 = 1000.0;
const PLAYER_SPEED: f32 = 100.0;
const PLAYER_JUMP_FORCE: f32 = 180.0;
const PLAYER_UNSTUCK_NUDGE_SPEED: f32 = 50.0;
const PLAYER_AIR_CONTROL: f32 = 0.25;
const PLAYER_AIR_FRICTION: f32 = 50.0;

const GRAVITY_SCALE: f32 = 500.0;
const TERMINAL_VELOCITY: f32 = 500.0;
const TICKRATE: f32 = 20.0;

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct TickTimer(Timer);

fn main() {
	App::new()
		.add_plugins(
			DefaultPlugins
				.set(WindowPlugin {
					window: WindowDescriptor {
						width: WINDOW_DEFAULT_WIDTH,
						height: WINDOW_DEFAULT_HEIGHT,
						resizable: true,
						title: String::from("framework"),
						..Default::default()
					},
					..default()
				})
				.set(ImagePlugin::default_nearest()),
		)
		.add_plugin(Inputs)
		.add_plugin(Grid)
		.add_plugin(PlayerPhysics)
		.add_plugin(TilePhysics)
		.add_plugin(Players)
		.add_plugin(SpritesPlugin)
		.add_plugin(DevTools)
		.add_startup_system(startup)
		.insert_resource(Settings {
			..Default::default()
		})
		.insert_resource(ClearColor(Color::rgb(0.30, 0.20, 0.10)))
		.insert_resource(TickTimer(Timer::from_seconds(
			1.0 / TICKRATE,
			TimerMode::Repeating,
		)))
		.run();
}

fn startup(mut commands: Commands, mut windows: ResMut<Windows>, sprites: Res<Sprites>) {
	let mut projection = OrthographicProjection::default();
	projection.scale = CAMERA_PROJECTION_SCALE;
	commands.spawn((
		Camera2dBundle {
			projection,
			..Default::default()
		},
		VisibilityBundle {
			..Default::default()
		},
		MainCamera,
	));

	commands.spawn((
		SpriteBundle {
			transform: Transform::from_xyz(0.0, 0.0, -100.0),
			texture: sprites.cursor.clone(),
			sprite: Sprite {
				..Default::default()
			},
			..default()
		},
		Cursor,
	));
	windows
		.get_primary_mut()
		.unwrap()
		.set_cursor_visibility(false);

	commands.spawn((
		SpriteBundle {
			transform: Transform::from_translation(Vec3::ZERO),
			texture: sprites.player.clone(),
			..Default::default()
		},
		PlayerBundle {
			..Default::default()
		},
	));

	/*
	commands.spawn((
		SpriteBundle {
			transform: Transform::from_translation(Vec3::ZERO),
			texture: asset_server.load("player.png"),
			..Default::default()
		},
		PlayerBundle {
			player: Player::Remote,
			..Default::default()
		},
	));
	*/
}
