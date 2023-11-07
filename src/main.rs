mod devtools;
mod grid;
mod inputs;
mod light;
mod playerphysics;
mod players;
mod settings;
mod sprites;
mod tileoutline;
mod tilephysics;
mod tiles;
mod tiletypes;
mod worldgen;

use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::{math::Vec3, window::Cursor};
use devtools::DevTools;
use grid::Grid;
use inputs::Inputs;
use light::Light;
use playerphysics::{PlayerPhysics, Position, Velocity};
use players::{Player, PlayerBundle, Players};
use settings::Settings;
use sprites::{setup_sprites, Sprites, SpritesPlugin};
use tilephysics::TilePhysics;

const WINDOW_DEFAULT_WIDTH: f32 = 1280.0;
const WINDOW_DEFAULT_HEIGHT: f32 = 720.0;

const CHUNK_SIZE: (u8, u8) = (32, 32);
const TILE_SIZE: UVec2 = UVec2::new(8, 8);
const RENDER_DISTANCE: UVec2 = UVec2::new(3, 2);
const UNRENDER_DISTANCE: UVec2 = UVec2::new(4, 3);
const CAMERA_PROJECTION_SCALE: f32 = 0.4;

const PLAYER_SIZE: UVec2 = UVec2::new(20, 36);
const PLAYER_ACCEL: f32 = 3000.0;
const PLAYER_SPEED: f32 = 100.0;
const PLAYER_JUMP_FORCE: f32 = 180.0;
const PLAYER_UNSTUCK_NUDGE_SPEED: f32 = 50.0;
const PLAYER_AIR_CONTROL: f32 = 0.10;
const PLAYER_AIR_FRICTION: f32 = 50.0;

const GRAVITY_SCALE: f32 = 500.0;
const TERMINAL_VELOCITY: f32 = 500.0;
const TICKRATE: f32 = 20.0;

#[derive(Component)]
struct WorldCursor;

#[derive(Component)]
struct UIWrapper;

#[derive(Component)]
struct ScreenCursor;

#[derive(Component)]
struct MainCamera;

#[derive(Resource)]
struct TickTimer(Timer, u64);

fn main() {
	App::new()
		.add_plugins(
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						resizable: true,
						title: String::from("bevy-tilegame-base"),
						cursor: Cursor {
							visible: false,
							..default()
						},
						resolution: WindowResolution::new(
							WINDOW_DEFAULT_WIDTH,
							WINDOW_DEFAULT_HEIGHT,
						),
						..default()
					}),
					..default()
				})
				.set(ImagePlugin::default_nearest()),
		)
		.add_event::<TickEvent>()
		.add_plugins(Inputs)
		.add_plugins(Grid)
		.add_plugins(PlayerPhysics)
		.add_plugins(TilePhysics)
		.add_plugins(Players)
		.add_plugins(SpritesPlugin)
		.add_plugins(Light)
		.add_plugins(DevTools)
		.add_systems(Startup, (setup_sprites, apply_deferred, startup).chain())
		.add_systems(Update, tick)
		.insert_resource(Settings {
			..Default::default()
		})
		.insert_resource(ClearColor(Color::rgb(0.30, 0.20, 0.10)))
		.insert_resource(TickTimer(
			Timer::from_seconds(1.0 / TICKRATE, TimerMode::Repeating),
			0,
		))
		.run();
}

fn startup(mut commands: Commands, sprites: Res<Sprites>) {
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

	commands.spawn((WorldCursor, Transform::from_translation(Vec3::ZERO)));

	commands.spawn((
		SpriteBundle {
			transform: Transform::from_translation(Vec3::new(50.0, -400.0, 10.0)),
			texture: sprites.player.clone(),
			..Default::default()
		},
		PlayerBundle {
			position: Position(Vec2::new(50.0, -400.0)),
			..Default::default()
		},
	));

	let uiwrapper = commands
		.spawn((
			NodeBundle {
				style: Style {
					position_type: PositionType::Absolute,
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					..Default::default()
				},
				..Default::default()
			},
			UIWrapper,
		))
		.id();

	let screencursor = commands
		.spawn((
			ScreenCursor,
			ImageBundle {
				style: Style {
					width: Val::Px(32.0),
					height: Val::Px(32.0),
					..default()
				},
				image: sprites.cursor.clone().into(),
				..default()
			},
		))
		.id();

	commands.entity(uiwrapper).add_child(screencursor);

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

fn tick(time: Res<Time>, mut timer: ResMut<TickTimer>, mut ev: EventWriter<TickEvent>) {
	if timer.0.tick(time.delta()).just_finished() {
		timer.1 += 1;
		ev.send(TickEvent(timer.1));
	}
}

#[derive(Event)]
pub struct TickEvent(u64);
