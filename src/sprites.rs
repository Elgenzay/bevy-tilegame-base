use std::fs::read_dir;

use bevy::{
	prelude::{App, AssetServer, Commands, Handle, Image, Plugin, Res, Resource},
	text::Font,
	utils::HashMap,
};

use crate::tiletypes::TileType;

pub struct SpritesPlugin;
impl Plugin for SpritesPlugin {
	fn build(&self, _app: &mut App) {
		//app.add_system(setup_sprites);
	}
}

pub fn setup_sprites(mut commands: Commands, asset_server: Res<AssetServer>) {
	let mut tiles = HashMap::new();
	for tile_type in TileType::all() {
		if !tile_type.is_visible() {
			continue;
		}
		let mut images = Vec::new();
		let tilename = tile_type.get_sprite_dir_name();
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

	let mut fonts = HashMap::new();
	fonts.insert(
		"pressstart2p".to_string(),
		asset_server.load("fonts/PressStart2P/PressStart2P-Regular.ttf"),
	);

	commands.insert_resource(Sprites {
		cursor: asset_server.load("cursor.png"),
		player: asset_server.load("player.png"),
		debugtilemarker: asset_server.load("debugtilemarker.png"),
		tiles,
		tile_outlines,
		fonts,
	});
}

#[derive(Resource)]
pub struct Sprites {
	pub cursor: Handle<Image>,
	pub player: Handle<Image>,
	pub debugtilemarker: Handle<Image>,
	pub tiles: HashMap<String, Vec<Handle<Image>>>,
	pub tile_outlines: Vec<Handle<Image>>,
	pub fonts: HashMap<String, Handle<Font>>,
}
