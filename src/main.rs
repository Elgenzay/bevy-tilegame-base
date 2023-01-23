mod grid;
mod inputs;
mod physics;
mod players;
mod settings;

use bevy::math::Vec3;
use bevy::prelude::*;
use bevy_ecs_tilemap::{prelude::TilemapTileSize, TilemapPlugin};
use grid::{spawn_chunk, Grid, Map};
use inputs::Inputs;
use physics::{Physics, Velocity};
use players::{Player, PlayerBundle, Players};
use settings::Settings;

const WINDOW_DEFAULT_WIDTH: f32 = 1280.0;
const WINDOW_DEFAULT_HEIGHT: f32 = 720.0;

const CHUNK_SIZE: UVec2 = UVec2 { x: 4, y: 4 };
const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };

const PLAYER_SIZE: UVec2 = UVec2 { x: 24, y: 24 };
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
		.add_plugin(TilemapPlugin)
		.add_plugin(Inputs)
		.add_plugin(Grid)
		.add_plugin(Physics)
		.add_plugin(Players)
		.add_startup_system(startup)
		.insert_resource(Settings {
			..Default::default()
		})
		.run();
}

fn startup(
	mut commands: Commands,
	mut windows: ResMut<Windows>,
	asset_server: Res<AssetServer>,
	mut map: ResMut<Map>,
) {
	commands.spawn((Camera2dBundle::default(), MainCamera));

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

	spawn_chunk(&mut commands, &asset_server, IVec2::new(0, 1), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(0, -2), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(-2, -2), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(-2, 0), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(2, 2), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(-1, -1), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(1, 0), &mut map);

	spawn_chunk(&mut commands, &asset_server, IVec2::new(-3, 0), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(-3, 1), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(-3, 2), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(-4, 0), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(-4, 1), &mut map);
	spawn_chunk(&mut commands, &asset_server, IVec2::new(-4, 2), &mut map);
}
