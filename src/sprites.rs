use crate::tiletypes::TileType;
use bevy::{
	prelude::{AssetServer, Commands, Handle, Image, Res, Resource},
	text::Font,
	utils::HashMap,
};
use std::fs::read_dir;
use strum::IntoEnumIterator;

pub fn setup_sprites(mut commands: Commands, asset_server: Res<AssetServer>) {
	let mut tiles = HashMap::new();

	for tile_type in TileType::iter() {
		if !tile_type.is_visible() {
			continue;
		}

		let mut images = vec![];
		let tilename = tile_type.to_string();

		let mut file_names: Vec<String> = read_dir(format!("assets/tiles/{tilename}"))
			.unwrap()
			.filter_map(|entry| entry.ok())
			.filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
			.collect();

		file_names.sort_by(|a, b| {
			let a_num: usize = a.trim_end_matches(".png").parse().unwrap();
			let b_num: usize = b.trim_end_matches(".png").parse().unwrap();
			a_num.cmp(&b_num)
		});

		for filename in file_names {
			images.push(asset_server.load(format!("tiles/{tilename}/{filename}")));
		}

		tiles.insert(tilename, images);
	}
	let mut tile_outlines = vec![];

	for n in 1..=47 {
		tile_outlines.push(asset_server.load(format!("tile_outlines/{n}.png")));
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
