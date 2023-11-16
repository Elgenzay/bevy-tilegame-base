use bevy::{
	prelude::{
		App, Color, Commands, Event, EventReader, EventWriter, Plugin, ResMut, Resource, Startup,
		Transform, Update, Vec2,
	},
	sprite::{Sprite, SpriteBundle},
	utils::{HashMap, HashSet},
};

use crate::{
	grid::{Coordinate, Map, MapTile},
	TILE_SIZE,
};

pub struct Light;

impl Plugin for Light {
	fn build(&self, app: &mut App) {
		app.add_event::<AddLightSourceEvent>()
			.add_event::<LightingUpdateEvent>()
			.add_systems(Update, (add_lightsource_event, lighting_update_event))
			.add_systems(Startup, initialize_lightsources);
	}
}

fn initialize_lightsources(mut commands: Commands) {
	commands.insert_resource(LightSources(HashMap::new()));
}

fn add_lightsource_event(
	mut event_add: EventReader<AddLightSourceEvent>,
	mut lightsources: ResMut<LightSources>,
	mut event_lightingupdate: EventWriter<LightingUpdateEvent>,
) {
	for ev in event_add.read() {
		lightsources.add_lightsource(ev.0, &mut event_lightingupdate);
	}
}

fn lighting_update_event(
	mut map: ResMut<Map>,
	mut ev_update_l: EventReader<LightingUpdateEvent>,
	mut lightsources: ResMut<LightSources>,
	mut commands: Commands,
) {
	for ev in ev_update_l.read() {
		if let Some(t) = map.get_tile(ev.0) {
			if !t.tile_type.is_emitter() {
				lighting_update(
					&mut lightsources,
					ev.0,
					&mut map,
					&mut commands,
					HashMap::new(),
				);
			}
		}
	}
}

fn lighting_update(
	lightsources: &mut LightSources,
	coord: Coordinate,
	map: &mut Map,
	commands: &mut Commands,
	mut checked_rays: HashMap<(i32, i32), HashSet<u16>>,
) -> HashMap<(i32, i32), HashSet<u16>> {
	let mut new_light_levels: HashMap<(i32, i32), u8> = HashMap::new();
	let mut c_vec = vec![coord];
	let mut updated_tiles = HashSet::new();
	while let Some(c) = c_vec.pop() {
		if lightsources.0.is_empty() {
			break;
		}
		for lightsource in lightsources.0.iter_mut() {
			let new_set = &HashSet::new();
			let rays = checked_rays.get(lightsource.0).unwrap_or(new_set);
			let update = lightsource.1.update_light_tile(map, c, rays);
			checked_rays.insert(*lightsource.0, rays.union(&update.1).cloned().collect());
			for (k, v) in update.0.iter() {
				if let Some(prev) = new_light_levels.get(k) {
					new_light_levels.insert(
						*k,
						if *prev > v.light_level {
							*prev
						} else {
							v.light_level
						},
					);
				} else {
					new_light_levels.insert(*k, v.light_level);
				};
				let new_c = Coordinate::Tile { x: k.0, y: k.1 };
				if new_c != c && !updated_tiles.contains(&(new_c.x_i32(), new_c.y_i32())) {
					updated_tiles.insert((new_c.x_i32(), new_c.y_i32()));
					c_vec.push(new_c);
				}
			}
		}
	}

	for (k, lvl) in new_light_levels.iter() {
		let coord = Coordinate::Tile { x: k.0, y: k.1 };
		if let Some(t) = map.get_tile(coord) {
			if t.light_level == *lvl {
				continue;
			}
		} else {
			continue;
		}

		if let Some(t) = map.get_tile_mut(coord) {
			t.light_level = *lvl;
		}

		if let Some(t) = map.get_tile(coord) {
			if let Some(mut e) = commands.get_entity(t.light_entity) {
				let color = Color::rgba_u8(0, 0, 0, u8::MAX - lvl);
				e.insert(SpriteBundle {
					sprite: Sprite {
						color,
						custom_size: Some(Vec2::new(TILE_SIZE.x as f32, TILE_SIZE.y as f32)),
						..Default::default()
					},
					transform: Transform::from_xyz(0.0, 0.0, 3.0),
					..Default::default()
				});
			}
		}
	}

	checked_rays
}

#[derive(Resource)]
struct LightSources(HashMap<(i32, i32), LightSource>);

impl LightSources {
	fn add_lightsource(
		&mut self,
		maptile: MapTile,
		ev_lighting_update: &mut EventWriter<LightingUpdateEvent>,
	) {
		println!("lightsource added");
		let coord = maptile.tile_coord;
		let emitter = if let Ok(e) = maptile.tile_type.get_emitter() {
			e
		} else {
			return;
		};
		let r_i32 = emitter.radius as i32;
		let mut ray_index: u16 = 0;
		let mut rays = HashMap::new();
		for x in -r_i32..=r_i32 {
			for y in -r_i32..=r_i32 {
				if x.abs() != r_i32 && y.abs() != r_i32 {
					continue;
				}
				ray_index += 1;
				let ray = coord
					.raycast_to(
						Coordinate::Tile { x, y }.moved(&Vec2::new(coord.x_f32(), coord.y_f32())),
					)
					.to_vec();
				rays.insert(ray_index, ray);
			}
		}
		println!("radius: {}", emitter.radius);
		println!("rays: {}", rays.len());

		let lightsource = LightSource::new(emitter, rays);
		ev_lighting_update.send(LightingUpdateEvent(maptile.tile_coord)); //todo delay
		self.0.insert((coord.x_i32(), coord.y_i32()), lightsource);
	}
}

struct LightSource {
	emitter: Emitter,
	rays: HashMap<u16, Vec<Coordinate>>,
	tiles: HashMap<(i32, i32), LightTile>,
}

impl LightSource {
	fn new(emitter: Emitter, rays: HashMap<u16, Vec<Coordinate>>) -> Self {
		let mut tiles: HashMap<(i32, i32), LightTile> = HashMap::new();
		for ray in rays.iter() {
			for c in ray.1 {
				let key = &(c.x_i32(), c.y_i32());
				if let Some(existing) = tiles.get(key) {
					let mut ray_indexes = existing.ray_indexes.clone();
					ray_indexes.push(*ray.0);
					tiles.insert(
						*key,
						LightTile {
							ray_indexes,
							..Default::default()
						},
					);
				} else {
					tiles.insert(
						*key,
						LightTile {
							ray_indexes: vec![*ray.0],
							..Default::default()
						},
					);
				}
			}
		}
		Self {
			emitter,
			rays,
			tiles,
		}
	}

	fn update_light_tile(
		&mut self,
		map: &Map,
		coord: Coordinate,
		checked: &HashSet<u16>,
	) -> (HashMap<(i32, i32), LightTile>, HashSet<u16>) {
		let mut updated_light_tiles: HashMap<(i32, i32), LightTile> = HashMap::new();
		let mut checked_rays = HashSet::new();

		if let Some(prev_light_tile) = self.tiles.get(&(coord.x_i32(), coord.y_i32())) {
			for index in prev_light_tile.ray_indexes.clone() {
				if checked.contains(&index) {
					continue;
				}
				checked_rays.insert(index);
				if let Some(r) = self.rays.get(&index) {
					#[allow(unused_mut)] //todo
					let mut obstructed = false;
					//let mut passed_target = false;
					let center = r.get(0).unwrap();
					for c in r {
						if let Some(maptile) = map.get_tile(*c) {
							//if *c == coord {
							//	passed_target = true;
							//}
							let k = (c.x_i32(), c.y_i32());
							if updated_light_tiles.get(&k).is_some() {
								//continue;
							};
							let distance = f32::sqrt(
								(center.x_f32() - c.x_f32()).powf(2.0)
									+ (center.y_f32() - c.y_f32()).powf(2.0),
							);

							let new_light_level = if obstructed {
								0
							} else {
								let r_f32 = self.emitter.radius as f32;
								(u8::MAX as f32 * ((r_f32 - distance) / r_f32)) as u8
							};

							//if new_light_level > prev_level {
							updated_light_tiles.insert(
								k,
								LightTile {
									distance,
									light_level: new_light_level,
									ray_indexes: prev_light_tile.ray_indexes.clone(),
								},
							);
							if maptile.tile_type.is_opaque() && !maptile.tile_type.is_emitter() {
								//obstructed = true;
							}
						}
					}
				}
			}
		}
		for (k, v) in updated_light_tiles.iter() {
			self.tiles.insert(*k, v.clone());
		}
		(updated_light_tiles, checked_rays)
	}
}

#[derive(Clone)]
struct LightTile {
	#[allow(dead_code)] //todo
	distance: f32,
	ray_indexes: Vec<u16>,
	light_level: u8,
}

impl Default for LightTile {
	fn default() -> Self {
		Self {
			distance: 0.0,
			ray_indexes: vec![],
			light_level: 0,
		}
	}
}

#[derive(Copy, Clone, PartialEq)]
pub struct Emitter {
	pub radius: u8,
	pub color: Option<Color>,
}

impl Default for Emitter {
	fn default() -> Self {
		Self {
			radius: 20,
			color: None,
		}
	}
}

#[derive(Event)]
pub struct AddLightSourceEvent(pub MapTile);

#[derive(Event)]
pub struct LightingUpdateEvent(pub Coordinate);
