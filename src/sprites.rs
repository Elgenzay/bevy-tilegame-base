use std::fs::read_dir;

use bevy::{
	prelude::{App, AssetServer, Commands, Handle, Image, Plugin, Res, Resource, StartupStage},
	utils::HashMap,
};

use crate::tiles::TileType;

pub struct SpritesPlugin;
impl Plugin for SpritesPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system_to_stage(StartupStage::PreStartup, setup);
	}
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	let mut tiles = HashMap::new();
	for tile_type in [
		TileType::DebugBrown,
		TileType::DebugGray,
		TileType::DebugGreen,
	] {
		let mut images = Vec::new();
		let tilename = tile_type.get_name();
		for entry in read_dir(format!("assets/tiles/{}", tilename)).unwrap() {
			let entry = entry.unwrap();
			if let Some(filename) = entry.file_name().to_str() {
				images.push(asset_server.load(format!("tiles/{}/{}", tilename, filename)));
			}
		}
		tiles.insert(tilename, images);
	}
	let mut tile_outlines = vec![];
	for x in 1..=47 {
		tile_outlines.push(asset_server.load(format!("tile_outlines/{}.png", x.to_string())));
	}

	commands.insert_resource(Sprites {
		cursor: asset_server.load("cursor.png"),
		player: asset_server.load("player.png"),
		tiles,
		tile_outlines,
	});
}

#[derive(Resource)]
pub struct Sprites {
	pub cursor: Handle<Image>,
	pub player: Handle<Image>,
	pub tiles: HashMap<String, Vec<Handle<Image>>>,
	pub tile_outlines: Vec<Handle<Image>>,
}
