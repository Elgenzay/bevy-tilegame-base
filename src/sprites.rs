use bevy::prelude::{
	App, AssetServer, Commands, Handle, Image, Plugin, Res, Resource, StartupStage,
};

pub struct SpritesPlugin;
impl Plugin for SpritesPlugin {
	fn build(&self, app: &mut App) {
		app.add_startup_system_to_stage(StartupStage::PreStartup, setup);
	}
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(Sprites {
		cursor: asset_server.load("cursor.png"),
		player: asset_server.load("player.png"),
	});
}

#[derive(Resource)]
pub struct Sprites {
	pub cursor: Handle<Image>,
	pub player: Handle<Image>,
}
