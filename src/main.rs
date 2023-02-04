mod grid;
mod inputs;
mod playerphysics;
mod players;
mod settings;
mod tilephysics;
mod worldgen;

use bevy::math::Vec3;
use bevy::prelude::*;
use grid::Grid;
use inputs::Inputs;
use playerphysics::{PlayerPhysics, Velocity};
use players::{Player, PlayerBundle, Players};
use settings::Settings;
use tilephysics::TilePhysics;

const WINDOW_DEFAULT_WIDTH: f32 = 1280.0;
const WINDOW_DEFAULT_HEIGHT: f32 = 720.0;

const CHUNK_SIZE: (u8, u8) = (32, 32);
const TILE_SIZE: UVec2 = UVec2::new(8, 8);
const RENDER_DISTANCE: UVec2 = UVec2::new(3, 2);
const UNRENDER_DISTANCE: UVec2 = UVec2::new(4, 3);

const PLAYER_SIZE: UVec2 = UVec2::new(20, 36);
const PLAYER_ACCEL: f32 = 1000.0;
const PLAYER_SPEED: f32 = 100.0;
const PLAYER_JUMP_FORCE: f32 = 200.0;
const AIR_FRICTION: f32 = 15.0;
const AIR_CONTROL: f32 = 0.1;
const GRAVITY_SCALE: f32 = 400.0;
const TERMINAL_VELOCITY: f32 = 500.0;

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct MainCamera;

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
		.add_startup_system(startup)
		.insert_resource(Settings {
			..Default::default()
		})
		.insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
		.run();
}

fn startup(mut commands: Commands, mut windows: ResMut<Windows>, asset_server: Res<AssetServer>) {
	let mut projection = OrthographicProjection::default();
	projection.scale = 0.5;
	commands.spawn((
		Camera2dBundle {
			projection,
			..Default::default()
		},
		MainCamera,
	));

	commands.spawn((
		SpriteBundle {
			transform: Transform::from_xyz(0.0, 0.0, -100.0),
			texture: asset_server.load("cursor.png"),
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
			texture: asset_server.load("player.png"),
			..Default::default()
		},
		PlayerBundle {
			..Default::default()
		},
	));

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
}
