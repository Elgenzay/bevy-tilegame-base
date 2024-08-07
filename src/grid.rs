use crate::{
	light::{AddLightSourceEvent, LightingUpdateEvent},
	playerphysics::{Collider, Position},
	players::Player,
	sprites::Sprites,
	tilephysics::UpdateTileEvent,
	tiles::{set_tile, set_tile_result},
	tiletypes::TileType,
	worldgen::tiletype_at,
	CHUNK_SIZE, RENDER_DISTANCE, TILE_SIZE, UNRENDER_DISTANCE,
};
use bevy::{
	prelude::{
		App, BuildChildren, Children, Commands, Component, Deref, DerefMut, DespawnRecursiveExt,
		Entity, Event, EventReader, EventWriter, IVec2, IntoSystemConfigs, Plugin, Query, Res,
		ResMut, Resource, Transform, TransformBundle, Update, Vec2, Vec3, VisibilityBundle, With,
	},
	utils::hashbrown::HashMap,
};
use bresenham::Bresenham;

pub struct Grid;

impl Plugin for Grid {
	fn build(&self, app: &mut App) {
		app.insert_resource(Map(HashMap::new()))
			.add_event::<DestroyTileEvent>()
			.add_event::<CreateTileEvent>()
			.add_systems(
				Update,
				(render_chunks, destroy_tile_event, create_tile_event).chain(),
			);
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
	pub fn get_tile(&self, coord: Coordinate) -> Option<MapTile> {
		let chunk_coord = coord.as_chunk_coord();
		let chunklocal_coord = coord.as_chunklocal_coord();

		if let Some(chunk) = self.0.get(&(chunk_coord.x_i32(), chunk_coord.y_i32())) {
			if let Some(tile) = chunk
				.tiles
				.get(&(chunklocal_coord.x_u8(), chunklocal_coord.y_u8()))
			{
				return Some(*tile);
			}
		}

		None
	}

	pub fn get_tile_mut(&mut self, coord: Coordinate) -> Option<&mut MapTile> {
		let chunk_coord = coord.as_chunk_coord();
		let chunklocal_coord = coord.as_chunklocal_coord();

		if let Some(chunk) = self.0.get_mut(&(chunk_coord.x_i32(), chunk_coord.y_i32())) {
			if let Some(tile) = chunk
				.tiles
				.get_mut(&(chunklocal_coord.x_u8(), chunklocal_coord.y_u8()))
			{
				return Some(tile);
			}
		}

		None
	}
}

pub struct MapChunk {
	pub entity: Entity,
	pub tiles: HashMap<(u8, u8), MapTile>,
}

#[derive(Clone, Copy)]
pub struct MapTile {
	pub tile_entity: Entity,
	pub outline_entity: Entity,
	pub sprite_entity: Entity,
	pub light_entity: Entity,
	pub light_level: u8,
	pub outline_id: usize,
	pub tile_type: TileType,
	pub tile_coord: Coordinate,
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
			Self::Tile { x, y: _ } => *x,
			Self::Chunk { x, y: _ } => *x,
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

	pub fn x_isize(&self) -> isize {
		match self {
			Self::World { x, y: _ } => *x as isize,
			Self::ChunkLocal { x, y: _ } => *x as isize,
			Self::Tile { x, y: _ } => *x as isize,
			Self::Chunk { x, y: _ } => *x as isize,
		}
	}

	pub fn y_isize(&self) -> isize {
		match self {
			Self::World { x: _, y } => *y as isize,
			Self::Tile { x: _, y } => *y as isize,
			Self::Chunk { x: _, y } => *y as isize,
			Self::ChunkLocal { x: _, y } => *y as isize,
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

	pub fn as_world_coord(&self) -> Self {
		let tile_coord = self.as_tile_coord();
		Self::World {
			x: tile_coord.x_f32() * TILE_SIZE.x as f32,
			y: tile_coord.y_f32() * TILE_SIZE.y as f32,
		}
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

	pub fn raycast_to(&self, other_tile_coord: Coordinate) -> Vec<Coordinate> {
		let mut maptiles = vec![];

		for (x, y) in Bresenham::new(
			(self.x_isize(), self.y_isize()),
			(other_tile_coord.x_isize(), other_tile_coord.y_isize()),
		) {
			maptiles.push(Coordinate::Tile {
				x: x as i32,
				y: y as i32,
			});
		}

		maptiles
	}
}

impl PartialEq for Coordinate {
	fn eq(&self, other: &Self) -> bool {
		self.x_i32() == other.x_i32() && self.y_i32() == other.y_i32()
	}
}

pub fn spawn_chunk(
	commands: &mut Commands,
	chunk_pos: IVec2,
	map: &mut Map,
	sprites: &Sprites,
	ev_update: &mut EventWriter<UpdateTileEvent>,
	ev_addlightsource: &mut EventWriter<AddLightSourceEvent>,
	ev_updatelighting: &mut EventWriter<LightingUpdateEvent>,
) -> Entity {
	let tilesize_x_f32 = TILE_SIZE.x as f32;
	let tilesize_y_f32 = TILE_SIZE.y as f32;

	let transform = Transform::from_translation(Vec3::new(
		chunk_pos.x as f32 * CHUNK_SIZE.0 as f32 * tilesize_x_f32,
		chunk_pos.y as f32 * CHUNK_SIZE.1 as f32 * tilesize_y_f32,
		0.0,
	));

	let chunk_entity = commands
		.spawn((
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
		))
		.id();

	if let Some(v) = map.0.get(&(chunk_pos.x, chunk_pos.y)) {
		if let Some(e) = commands.get_entity(v.entity) {
			e.despawn_recursive();
		}
	}

	let mut tiles = HashMap::new();

	for x in 0..CHUNK_SIZE.0 {
		for y in 0..CHUNK_SIZE.1 {
			let x_f32 = x as f32;
			let y_f32 = y as f32;

			let tile = commands
				.spawn((
					VisibilityBundle {
						..Default::default()
					},
					TransformBundle {
						local: Transform {
							translation: Vec3 {
								x: x_f32 * tilesize_x_f32,
								y: y_f32 * tilesize_y_f32,
								z: 0.0,
							},
							..Default::default()
						},
						..Default::default()
					},
					Region::from_size(
						&Vec2::new(
							(x_f32 * TILE_SIZE.x as f32)
								+ (chunk_pos.x as f32 * CHUNK_SIZE.0 as f32 * TILE_SIZE.x as f32)
								- (tilesize_x_f32 * 0.5),
							(y_f32 * tilesize_y_f32)
								+ (chunk_pos.y as f32 * CHUNK_SIZE.1 as f32 * TILE_SIZE.y as f32)
								- (TILE_SIZE.y as f32 * 0.5),
						),
						&Vec2::new(tilesize_x_f32, tilesize_y_f32),
					),
				))
				.id();

			let outline = commands.spawn_empty().id();
			let sprite = commands.spawn_empty().id();
			let light = commands.spawn_empty().id();
			commands.entity(tile).add_child(outline);
			commands.entity(tile).add_child(sprite);
			commands.entity(tile).add_child(light);
			commands.entity(chunk_entity).add_child(tile);

			tiles.insert(
				(x, y),
				MapTile {
					tile_entity: tile,
					sprite_entity: sprite,
					outline_entity: outline,
					light_entity: light,
					light_level: 0,
					outline_id: 40,
					tile_type: TileType::Empty,
					tile_coord: Coordinate::Tile {
						x: (chunk_pos.x * CHUNK_SIZE.0 as i32) + x as i32,
						y: (chunk_pos.y * CHUNK_SIZE.1 as i32) + y as i32,
					},
				},
			);
		}
	}
	map.0.insert(
		(chunk_pos.x, chunk_pos.y),
		MapChunk {
			entity: chunk_entity,
			tiles,
		},
	);
	for x in 0..CHUNK_SIZE.0 {
		for y in 0..CHUNK_SIZE.1 {
			let tile_x = (chunk_pos.x * CHUNK_SIZE.0 as i32) + x as i32;
			let tile_y = (chunk_pos.y * CHUNK_SIZE.1 as i32) + y as i32;
			let tile_type = tiletype_at(tile_x, tile_y);

			if set_tile_result(
				commands,
				Coordinate::Tile {
					x: tile_x,
					y: tile_y,
				},
				tile_type,
				sprites,
				map,
				ev_update,
				ev_addlightsource,
				ev_updatelighting,
				None,
			)
			.is_err()
			{
				println!("Chunk load error");
			};
		}
	}

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
					ev_update.send(UpdateTileEvent(Coordinate::Tile {
						x: chunk_coord.x_i32() * CHUNK_SIZE.0 as i32 + x as i32,
						y: chunk_coord.y_i32() * CHUNK_SIZE.1 as i32 + y as i32,
					}));
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

			if !regions_overlap(tile_region, region) {
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

fn destroy_tile_event(
	mut ev_destroy: EventReader<DestroyTileEvent>,
	mut ev_update: EventWriter<UpdateTileEvent>,
	mut ev_addlightsource: EventWriter<AddLightSourceEvent>,
	mut ev_updatelighting: EventWriter<LightingUpdateEvent>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	sprites: Res<Sprites>,
) {
	for ev in ev_destroy.read() {
		set_tile(
			&mut commands,
			ev.0,
			TileType::Empty,
			&sprites,
			&mut map,
			&mut ev_update,
			&mut ev_addlightsource,
			&mut ev_updatelighting,
			None,
		);
	}
}

pub fn render_chunks(
	q_player: Query<(&Player, &Position)>,
	mut map: ResMut<Map>,
	mut commands: Commands,
	q_chunks: Query<&Chunk>,
	sprites: Res<Sprites>,
	mut ev_update: EventWriter<UpdateTileEvent>,
	mut ev_addlightsource: EventWriter<AddLightSourceEvent>,
	mut ev_updatelighting: EventWriter<LightingUpdateEvent>,
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
						&mut ev_addlightsource,
						&mut ev_updatelighting,
					);
				}
			}
		}
	}
}

pub fn create_tile_event(
	mut map: ResMut<Map>,
	mut commands: Commands,
	mut ev_create: EventReader<CreateTileEvent>,
	mut ev_update: EventWriter<UpdateTileEvent>,
	mut ev_addlightsource: EventWriter<AddLightSourceEvent>,
	mut ev_updatelighting: EventWriter<LightingUpdateEvent>,
	sprites: Res<Sprites>,
) {
	for ev in ev_create.read() {
		if let Some(prev_maptile) = ev.prev_maptile {
			if let Some(t) = map.get_tile(ev.coord) {
				if t.tile_type != prev_maptile.tile_type {
					continue;
				}
			} else {
				continue;
			}
		}

		set_tile(
			&mut commands,
			ev.coord,
			ev.new_tile_type,
			&sprites,
			&mut map,
			&mut ev_update,
			&mut ev_addlightsource,
			&mut ev_updatelighting,
			None,
		);
	}
}

pub fn xorshift_from_coord(coord: Coordinate) -> i32 {
	let mut i = coord.x_i32() * coord.y_i32();

	if i == 0 {
		i = coord.x_i32() + coord.y_i32();
	}

	// George Marsaglia's Xorshift
	i ^= i << 13;
	i ^= i >> 17;
	i ^= i << 5;

	i / 10
}

#[derive(Event)]
pub struct DestroyTileEvent(pub Coordinate);

#[derive(Event)]
pub struct CreateTileEvent {
	pub coord: Coordinate,
	pub new_tile_type: TileType,
	pub prev_maptile: Option<MapTile>,
}

impl CreateTileEvent {
	pub fn new(coord: Coordinate, new_tile_type: TileType, prev_maptile: Option<MapTile>) -> Self {
		Self {
			coord,
			new_tile_type,
			prev_maptile,
		}
	}
}
