mod cli;
mod config;
mod env;

use crate::cli::*;
use crate::config::Config;
use asteroids::{
	Element,
	client::ClientState,
	custom::ElementTrait,
	elements::{Model, PlaySpace, SkyLight, SkyTexture, Spatial},
	util::Migrate,
};
use clap::{Parser, Subcommand};
use env::{Environment, Node, NodeType};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{spatial::Transform, values::ResourceID};
use std::{
	fs::DirEntry,
	path::{Path, PathBuf},
	sync::OnceLock,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
	List,
	Install { path: PathBuf },
	SetDefault { env_name: String },
	Show { env_name: Option<String> },
}

fn main() {
	let args = Cli::parse();
	match args.command {
		Commands::List => list(),
		Commands::Install { path } => install(path),
		Commands::SetDefault { env_name } => {
			let config: Config = confy::load("atmosphere", "atmosphere").unwrap();
			set_default(config, env_name)
		}
		Commands::Show { .. } => show(),
	}
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
	path: PathBuf,
	#[serde(skip)]
	env: OnceLock<Environment>,
}
impl Migrate for State {
	type Old = Self;
}
impl ClientState for State {
	const QUALIFIER: &'static str = "org";
	const ORGANIZATION: &'static str = "stardustxr";
	const NAME: &'static str = "atmoshpere";

	fn initial_state_update(&mut self) {
		if let Commands::Show { env_name } = Cli::parse().command {
			let config: Config = confy::load("atmosphere", "atmosphere").unwrap();
			let env_path = env_name
				.as_ref()
				.map(Path::new)
				.unwrap_or(&config.environment);
			let data_path = environments_dir().join(env_path);
			self.path = data_path;
		} else {
			println!("somehow ran initial_state_update without using the show command")
		}
	}

	fn reify(&self) -> asteroids::Element<Self> {
		let env = self
			.env
			.get_or_init(|| Environment::load(self.path.join("env.kdl"), &self.path));
		let sky_light = env
			.sky_light
			.clone()
			.map(|v| SkyLight(ResourceID::Direct(v)).build());
		let sky_tex = env
			.sky_tex
			.clone()
			.map(|v| SkyTexture(ResourceID::Direct(v)).build());
		PlaySpace.with_children(
			[reify_node(&env.root)]
				.into_iter()
				.chain(sky_light)
				.chain(sky_tex),
		)
	}
}

fn reify_node(node: &Node) -> Element<State> {
	let node_type = &node.node_type;
	let children = node.children.iter().map(reify_node);
	match node_type {
		NodeType::Spatial => Spatial::default()
			.zoneable(true)
			.transform(node.transform)
			.with_children(children)
			.identify(&node.uuid),

		NodeType::Model(path_buf) => match Model::direct(path_buf) {
			Err(err) => {
				println!(
					"Error while loading model: {err}, from: {}",
					path_buf.to_string_lossy()
				);
				return Spatial::default()
					.zoneable(true)
					.transform(node.transform)
					.with_children(children);
			}
			Ok(v) => v.transform(node.transform).with_children(children),
		},

		NodeType::Box(scale) => Spatial::default()
			.zoneable(true)
			.transform({
				let scale = node.transform.scale.map(Vec3::from).unwrap_or(Vec3::ONE) * *scale;
				Transform {
					scale: Some(scale.into()),
					..node.transform
				}
			})
			.with_children(children),
	}
	.identify(&node.uuid)
}

#[tokio::main(flavor = "current_thread")]
async fn show() {
	asteroids::client::run::<State>(&[]).await
}

#[inline]
pub fn environments_dir() -> PathBuf {
	let dir = dirs::data_local_dir().unwrap().join("xr_environments");
	if !dir.exists() {
		let _ = std::fs::create_dir_all(&dir);
	}
	dir
}

pub fn get_list() -> Vec<DirEntry> {
	environments_dir()
		.read_dir()
		.unwrap()
		.filter_map(|dir| dir.ok())
		.filter(|dir| !dir.file_type().unwrap().is_file())
		// .filter(|dir| dir.path().join("env.kdl").exists())
		.collect::<Vec<_>>()
}
