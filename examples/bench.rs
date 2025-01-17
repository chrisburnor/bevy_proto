use std::time::Instant;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_proto::{HandlePath, ProtoCommands, ProtoComponent, ProtoData, ProtoPlugin, Prototypical};

const ENTITY_COUNT: u128 = 200_000;
const BATCH_SIZE: u128 = 5_000;
const BATCH_COUNT: u128 = ENTITY_COUNT / BATCH_SIZE;

fn spawn_sprites_proto(
	mut commands: Commands,
	data: Res<ProtoData>,
	asset_server: Res<AssetServer>,
) {
	println!("Spawning via Prototype:");
	let mut total: u128 = 0;
	let mut before = Instant::now();
	let proto = data.get_prototype("Sprite Test").expect("Should exist!");

	for _ in 0..BATCH_COUNT {
		for _ in 0..BATCH_SIZE {
			proto.spawn(&mut commands, &data, &asset_server);
		}
		println!("Prototype Batch: {:.2?}", before.elapsed());
		total += before.elapsed().as_millis();
		before = Instant::now();
	}

	println!(
		"Prototypes: {}ms for {} (avg. batch {}ms)",
		total,
		ENTITY_COUNT,
		total / BATCH_COUNT
	);
}

fn spawn_sprites_programmatic(
	mut commands: Commands,
	mut materials: ResMut<Assets<ColorMaterial>>,
	asset_server: Res<AssetServer>,
) {
	println!("Spawning Programmatically:");
	let mut total: u128 = 0;
	let mut before = Instant::now();
	let texture: Handle<Texture> = asset_server.load("textures/sprite.png");
	let mat = materials.add(texture.into());

	for _ in 0..BATCH_COUNT {
		for _ in 0..BATCH_SIZE {
			commands.spawn_bundle(SpriteBundle {
				material: mat.clone(),
				..Default::default()
			});
		}
		println!("Programmatic Batch: {:.2?}", before.elapsed());
		total += before.elapsed().as_millis();
		before = Instant::now();
	}

	println!(
		"Programmatic: {}ms for {} (avg. batch {}ms)",
		total,
		ENTITY_COUNT,
		total / BATCH_COUNT
	);
}

fn main() {
	println!(
		"Entity Count: {} | Batch Size: {}",
		ENTITY_COUNT, BATCH_SIZE
	);
	App::build()
		.add_plugins(DefaultPlugins)
		.add_plugin(ProtoPlugin::default())
		.add_startup_system(spawn_sprites_proto.system().label("prototype"))
		.add_startup_system(spawn_sprites_programmatic.system().after("prototype"))
		.run();
}

/// The code below is covered in the `bundles` example. It's an implementation
/// detail we don't need to focus on for this particular example

#[derive(Serialize, Deserialize)]
struct SpriteBundleDef {
	pub texture_path: HandlePath,
}

#[typetag::serde]
impl ProtoComponent for SpriteBundleDef {
	fn insert_self(&self, commands: &mut ProtoCommands, _asset_server: &Res<AssetServer>) {
		// === Get Prepared Assets === //
		let material: Handle<ColorMaterial> = commands
			.get_handle(self, &self.texture_path)
			.expect("Expected ColorMaterial handle to have been created");

		// === Generate Bundle === //
		let my_bundle = SpriteBundle {
			material,
			..Default::default()
		};

		// === Insert Generated Bundle === //
		commands.insert_bundle(my_bundle);
	}

	fn prepare(&self, world: &mut World, prototype: &Box<dyn Prototypical>, data: &mut ProtoData) {
		// === Load Handles === //
		let asset_server = world.get_resource::<AssetServer>().unwrap();
		let texture: Handle<Texture> = asset_server.load(self.texture_path.as_str());

		// === Transform Handles === //
		let mut mat_res = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
		let mat = mat_res.add(texture.into());

		// === Save Handles === //
		data.insert_handle(prototype, self, &self.texture_path, mat);
	}
}
