use crate::{
	playerphysics::{Collider, Position},
	players::Player,
	sprites::Sprites,
	tilephysics::UpdateTileEvent,
	tiles::{create_tile, WeightedTile},
	worldgen::tiletype_at,
	CHUNK_SIZE, RENDER_DISTANCE, TILE_SIZE, UNRENDER_DISTANCE,
};
use bevy::{
	prelude::{
		App, BuildChildren, Children, Commands, Component, Deref, DerefMut, DespawnRecursiveExt,
		Entity, EventReader, EventWriter, IVec2, Plugin, Query, Res, ResMut, Resource, Transform,
		Vec2, Vec3, VisibilityBundle, With,
	},
	transform::TransformBundle,
	utils::hashbrown::HashMap,
};

pub struct Grid;

impl Plugin for Grid {
	fn build(&self, app: &mut App) {
		app.insert_resource(Map(HashMap::new()))
			.add_event::<DestroyTileEvent>()
			.add_system(render_chunks)
			.add_system(destroy_tile);
	}
}

#[derive(Component)]
pub struct Chunk(IVec2);

#[derive(Component)]
pub struct Region {
	pub top: f32,
	pub left: f32,
	pub bottom: f32,
	pub right: f32,
}

impl Region {
	pub fn from_size(position: &Vec2, size: &Vec2) -> Self {
		Self {
			top: position.y + size.y,
			left: position.x,
			bottom: position.y,
			right: position.x + size.x,
		}
	}

	pub fn moved(&self, movement: &Vec2) -> Self {
		Self {
			top: self.top + movement.y,
			bottom: self.bottom + movement.y,
			left: self.left + movement.x,
			right: self.right + movement.x,
		}
	}
}

#[derive(Resource, Deref, DerefMut)]
pub struct Map(HashMap<(i32, i32), MapChunk>);

impl Map {
	pub fn get_tile(&self, coord: Coordinate) -> Result<Option<&MapTile>, ()> {
		match coord {
			Coordinate::Chunk { x: _, y: _ } => {
				panic!("Chunk coordinate passed to get_tile() instead of get_tiles()")
			}
			Coordinate::ChunkLocal { x: _, y: _ } => {
				panic!("ChunkLocal coordinate passed to get_tile()")
			}
			_ => (),
		};
		let chunk_coord = coord.as_chunk_coord();
		if let Some(map_chunk) = self.0.get(&(chunk_coord.x_i32(), chunk_coord.y_i32())) {
			let local_coord = coord.as_chunklocal_coord();
			if let Some(tile) = map_chunk
				.tiles
				.get(&(local_coord.x_u8(), local_coord.y_u8()))
			{
				return Ok(Some(tile));
			}
			return Ok(None);
		}
		Err(()) // chunk not loaded
	}

	pub fn set_tile(
		&mut self,
		commands: &mut Commands,
		coord: Coordinate,
		tile: Option<MapTile>,
	) -> Result<(), ()> {
		let chunk_coord = coord.as_chunk_coord();
		let chunk = match self.0.get_mut(&(chunk_coord.x_i32(), chunk_coord.y_i32())) {
			Some(v) => v,
			None => return Err(()), // chunk not loaded
		};
		let chunklocal = coord.as_chunklocal_coord();
		if let Some(tile) = chunk.tiles.remove(&(chunklocal.x_u8(), chunklocal.y_u8())) {
			if let Some(e) = commands.get_entity(tile.entity) {
				e.despawn_recursive();
			}
		}
		if let Some(tile) = tile {
			commands.entity(chunk.entity).add_child(tile.entity);
			chunk
				.tiles
				.insert((chunklocal.x_u8(), chunklocal.y_u8()), tile);
		} else {
			chunk.tiles.remove(&(chunklocal.x_u8(), chunklocal.y_u8()));
		}
		Ok(())
	}
}

pub struct MapChunk {
	entity: Entity,
	tiles: HashMap<(u8, u8), MapTile>,
}

#[derive(Clone, Copy)]
pub struct MapTile {
	pub entity: Entity,
	pub outline: Entity,
}

#[derive(Clone, Copy)]
pub enum Coordinate {
	World { x: f32, y: f32 },
	Tile { x: i32, y: i32 },
	Chunk { x: i32, y: i32 },
	ChunkLocal { x: u8, y: u8 },
}

impl Coordinate {
	pub fn x_i32(&self) -> i32 {
		match self {
			Self::World { x, y: _ } => *x as i32,
			Self::ChunkLocal { x, y: _ } => *x as i32,
			Self::Tile { x, y: _ } => *x as i32,
			Self::Chunk { x, y: _ } => *x as i32,
		}
	}

	pub fn y_i32(&self) -> i32 {
		match self {
			Self::World { x: _, y } => *y as i32,
			Self::Tile { x: _, y } => *y,
			Self::Chunk { x: _, y } => *y,
			Self::ChunkLocal { x: _, y } => *y as i32,
		}
	}

	pub fn x_u8(&self) -> u8 {
		match self {
			Self::ChunkLocal { x, y: _ } => *x,
			_ => panic!("Tried to get a non-chunklocal coord as u8"),
		}
	}

	pub fn y_u8(&self) -> u8 {
		match self {
			Self::ChunkLocal { x: _, y } => *y,
			_ => panic!("Tried to get a non-chunklocal coord as u8"),
		}
	}

	pub fn x_f32(&self) -> f32 {
		match self {
			Self::World { x, y: _ } => *x,
			Self::Tile { x, y: _ } => *x as f32,
			Self::Chunk { x, y: _ } => *x as f32,
			Self::ChunkLocal { x, y: _ } => *x as f32,
		}
	}

	pub fn y_f32(&self) -> f32 {
		match self {
			Self::World { x: _, y } => *y,
			Self::Tile { x: _, y } => *y as f32,
			Self::Chunk { x: _, y } => *y as f32,
			Self::ChunkLocal { x: _, y } => *y as f32,
		}
	}

	pub fn world_coord_from_vec2(v: Vec2) -> Self {
		Self::World { x: v.x, y: v.y }
	}

	pub fn as_tile_coord(&self) -> Self {
		match self {
			Self::World { x, y } => Self::Tile {
				x: ((x - (TILE_SIZE.x as f32 * 0.5)) / TILE_SIZE.x as f32).ceil() as i32,
				y: ((y - (TILE_SIZE.y as f32 * 0.5)) / TILE_SIZE.y as f32).ceil() as i32,
			},
			Self::Chunk { x: _, y: _ } => {
				panic!("Tried to convert chunk coordinate to tile coordinate")
			}
			Self::ChunkLocal { x: _, y: _ } => {
				panic!("Tried to convert chunklocal coordinate to tile coordinate")
			}
			Self::Tile { x: _, y: _ } => *self,
		}
	}

	pub fn as_chunk_coord(&self) -> Coordinate {
		let chunksize_x_f32 = CHUNK_SIZE.0 as f32;
		let chunksize_y_f32 = CHUNK_SIZE.1 as f32;
		match self {
			Coordinate::Tile { x, y } => Coordinate::Chunk {
				x: (*x as f32 / chunksize_x_f32).floor() as i32,
				y: (*y as f32 / chunksize_y_f32).floor() as i32,
			},
			Coordinate::World { x: _, y: _ } => {
				let tile_coord = self.as_tile_coord();
				Coordinate::Chunk {
					x: (tile_coord.x_f32() / chunksize_x_f32).floor() as i32,
					y: (tile_coord.y_f32() / chunksize_y_f32).floor() as i32,
				}
			}
			Coordinate::Chunk { x: _, y: _ } => *self,
			Coordinate::ChunkLocal { x: _, y: _ } => {
				panic!("Tried to convert chunklocal coordinate to chunk coordinate")
			}
		}
	}

	pub fn as_chunklocal_coord(&self) -> Self {
		let tile_coord = self.as_tile_coord();
		let chunk_size_x_i32 = CHUNK_SIZE.0 as i32;
		let chunk_size_y_i32 = CHUNK_SIZE.1 as i32;
		let x = match tile_coord.x_i32().checked_rem(chunk_size_x_i32) {
			Some(v) => {
				if v < 0 {
					v + chunk_size_x_i32
				} else {
					v
				}
			}
			None => 0,
		};
		let y = match tile_coord.y_i32().checked_rem(chunk_size_y_i32) {
			Some(v) => {
				if v < 0 {
					v + chunk_size_y_i32
				} else {
					v
				}
			}
			None => 0,
		};
		Self::ChunkLocal {
			x: x as u8,
			y: y as u8,
		}
	}

	pub fn moved(&self, movement: &Vec2) -> Coordinate {
		match self {
			Coordinate::World { x, y } => Coordinate::World {
				x: x + movement.x,
				y: y + movement.y,
			},
			Coordinate::Tile { x, y } => Coordinate::Tile {
				x: x + movement.x as i32,
				y: y + movement.y as i32,
			},
			Coordinate::Chunk { x, y } => Coordinate::Chunk {
				x: x + movement.x as i32,
				y: y + movement.y as i32,
			},
			Coordinate::ChunkLocal { x: _, y: _ } => panic!("Tried to move ChunkLocal coordinate"),
		}
	}
}

impl PartialEq for Coordinate {
	fn eq(&self, other: &Self) -> bool {
		if self.x_i32() == other.x_i32() && self.y_i32() == other.y_i32() {
			true
		} else {
			false
		}
	}
}

pub fn spawn_chunk(
	commands: &mut Commands,
	chunk_pos: IVec2,
	map: &mut Map,
	sprites: &Sprites,
	ev_update: &mut EventWriter<UpdateTileEvent>,
) -> Entity {
	let chunk_entity = commands.spawn_empty().id();
	let mut mapchunk_tiles = HashMap::new();
	let tilesize_x_f32 = TILE_SIZE.x as f32;
	let tilesize_y_f32 = TILE_SIZE.y as f32;
	for x in 0..CHUNK_SIZE.0 {
		for y in 0..CHUNK_SIZE.1 {
			let tile_x = (chunk_pos.x * CHUNK_SIZE.0 as i32) + x as i32;
			let tile_y = (chunk_pos.y * CHUNK_SIZE.1 as i32) + y as i32;
			let tile_type = match tiletype_at(tile_x, tile_y) {
				Some(v) => v,
				None => continue,
			};
			let tile = create_tile(
				commands,
				Coordinate::Tile {
					x: tile_x,
					y: tile_y,
				},
				tile_type,
				&sprites,
			);
			if tile_type.is_weighted() {
				commands.entity(tile.entity).insert(WeightedTile {
					granularity: tile_type.get_granularity(),
					liquid: tile_type.is_liquid(),
				});
			}
			commands.entity(chunk_entity).add_child(tile.entity);
			mapchunk_tiles.insert((x, y), tile);
			ev_update.send(UpdateTileEvent(tile));
		}
	}

	let transform = Transform::from_translation(Vec3::new(
		chunk_pos.x as f32 * CHUNK_SIZE.0 as f32 * tilesize_x_f32,
		chunk_pos.y as f32 * CHUNK_SIZE.1 as f32 * tilesize_y_f32,
		0.0,
	));

	commands.entity(chunk_entity).insert((
		VisibilityBundle {
			..Default::default()
		},
		TransformBundle {
			local: transform,
			..Default::default()
		},
		Chunk(chunk_pos),
		Region::from_size(
			&Vec2::new(
				transform.translation.x - (tilesize_x_f32 * 0.5),
				transform.translation.y - (tilesize_y_f32 * 0.5),
			),
			&Vec2::new(
				CHUNK_SIZE.0 as f32 * tilesize_x_f32,
				CHUNK_SIZE.1 as f32 * tilesize_y_f32,
			),
		),
	));
	if let Some(v) = map.0.get(&(chunk_pos.x, chunk_pos.y)) {
		if let Some(e) = commands.get_entity(v.entity) {
			e.despawn_recursive();
		}
	}
	map.0.insert(
		(chunk_pos.x, chunk_pos.y),
		MapChunk {
			entity: chunk_entity,
			tiles: mapchunk_tiles,
		},
	);

	for x in -1..=1 {
		for y in -1..=1 {
			let chunk_coord = Coordinate::Chunk { x, y }.moved(&chunk_pos.as_vec2());
			let cs_offset_x = CHUNK_SIZE.0 - 1;
			let cs_offset_y = CHUNK_SIZE.1 - 1;
			let (x_min, x_max, y_min, y_max) = match x {
				-1 => match y {
					-1 => (cs_offset_x, cs_offset_x, cs_offset_y, cs_offset_y), // bottom left
					0 => (cs_offset_x, cs_offset_x, 0, cs_offset_y),            // left
					1 => (cs_offset_x, cs_offset_x, 0, 0),                      // top left
					_ => panic!(),
				},
				0 => match y {
					-1 => (0, cs_offset_x, cs_offset_y, cs_offset_y), // bottom
					0 => continue,                                    //
					1 => (0, cs_offset_x, 0, 0),                      // top
					_ => panic!(),
				},
				1 => match y {
					-1 => (0, 0, cs_offset_y, cs_offset_y), // bottom right
					0 => (0, 0, 0, cs_offset_y),            // right
					1 => (0, 0, 0, 0),                      // top right
					_ => panic!(),
				},
				_ => panic!(),
			};
			for x in x_min..=x_max {
				for y in y_min..=y_max {
					let tile_coord = Coordinate::Tile {
						x: chunk_coord.x_i32() * CHUNK_SIZE.0 as i32 + x as i32,
						y: chunk_coord.y_i32() * CHUNK_SIZE.1 as i32 + y as i32,
					};
					if let Ok(opt) = map.get_tile(tile_coord) {
						if let Some(t) = opt {
							ev_update.send(UpdateTileEvent(*t));
						}
					};
				}
			}
		}
	}

	chunk_entity
}

pub fn despawn_chunk(commands: &mut Commands, chunk_pos: IVec2, map: &mut Map) {
	if let Some(v) = map.0.get(&(chunk_pos.x, chunk_pos.y)) {
		if let Some(e) = commands.get_entity(v.entity) {
			e.despawn_recursive();
		};
	}
	map.0.remove(&(chunk_pos.x, chunk_pos.y));
}

pub fn region_collides(
	region: &Region,
	q_colliders: &Query<&Region, With<Collider>>,
	q_chunks: &Query<(&Region, &Children), With<Chunk>>,
) -> bool {
	for (chunk_region, chunk_children) in q_chunks {
		if !regions_overlap(chunk_region, region) {
			continue;
		}
		for &child in chunk_children.iter() {
			let tile_region = match q_colliders.get(child) {
				Ok(v) => v,
				Err(_) => continue,
			};
			if !regions_overlap(&tile_region, region) {
				continue;
			}
			return true;
		}
	}
	false
}

fn regions_overlap(region_1: &Region, region_2: &Region) -> bool {
	if region_1.right < region_2.left {
		return false;
	}
	if region_1.left > region_2.right {
		return false;
	}
	if region_1.top < region_2.bottom {
		return false;
	}
	if region_1.bottom > region_2.top {
		return false;
	}
	true
}

fn destroy_tile(
	mut ev_destroy: EventReader<DestroyTileEvent>,
	mut ev_update: EventWriter<UpdateTileEvent>,
	mut map: ResMut<Map>,
	mut commands: Commands,
) {
	for ev in ev_destroy.iter() {
		let chunk_coord = ev.0.as_chunk_coord();
		if let Some(mapchunk) = map.0.get_mut(&(chunk_coord.x_i32(), chunk_coord.y_i32())) {
			let local_coord = ev.0.as_chunklocal_coord();
			if let Some(tile) = mapchunk
				.tiles
				.remove(&(local_coord.x_u8(), local_coord.y_u8()))
			{
				if let Some(e) = commands.get_entity(tile.entity) {
					e.despawn_recursive();
				}
			}
		}
		for x in -1..=1 {
			for y in -1..=1 {
				if x == 0 && y == 0 {
					continue;
				}
				let c = ev.0.as_tile_coord().moved(&Vec2::new(x as f32, y as f32));
				if let Ok(opt) = map.get_tile(c) {
					if let Some(t) = opt {
						ev_update.send(UpdateTileEvent(*t));
					}
				} else {
					//unloaded chunk
				}
			}
		}
	}
}

pub fn render_chunks(
	q_player: Query<(&Player, &Position)>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	q_chunks: Query<&Chunk>,
	sprites: Res<Sprites>,
	mut ev_update: EventWriter<UpdateTileEvent>,
) {
	for (player, position) in q_player.iter() {
		if let Player::Local = player {
			//despawn
			let player_chunk_ivec2 = Coordinate::world_coord_from_vec2(position.0).as_chunk_coord();
			for chunk in q_chunks.iter() {
				if (chunk.0.x - player_chunk_ivec2.x_i32()).abs() > UNRENDER_DISTANCE.x as i32
					|| (chunk.0.y - player_chunk_ivec2.y_i32()).abs() > UNRENDER_DISTANCE.y as i32
				{
					despawn_chunk(&mut commands, chunk.0, &mut map);
				}
			}
			//spawn
			let current_chunk_coord =
				Coordinate::world_coord_from_vec2(position.0).as_chunk_coord();
			for x in (current_chunk_coord.x_i32() - RENDER_DISTANCE.x as i32)
				..(current_chunk_coord.x_i32() + RENDER_DISTANCE.x as i32)
			{
				for y in (current_chunk_coord.y_i32() - RENDER_DISTANCE.y as i32)
					..(current_chunk_coord.y_i32() + RENDER_DISTANCE.y as i32)
				{
					if map.0.contains_key(&(x, y)) {
						continue;
					}
					spawn_chunk(
						&mut commands,
						IVec2::new(x, y),
						&mut map,
						&sprites,
						&mut ev_update,
					);
				}
			}
		}
	}
}

pub struct DestroyTileEvent(pub Coordinate);
