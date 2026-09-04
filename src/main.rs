mod cli;
mod config;
mod env;

use crate::cli::*;
use crate::config::Config;
use clap::{Parser, Subcommand};
use env::{Environment, Node, NodeType};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use stardust_xr_asteroids::{
	Context, CustomElement, DynamicElement, Element, Entity, Migrate, Reify, Tasker, Transformable,
	client::ClientState,
	components::{Container, Poseable},
	elements::{Model, SkyLight, SkyTex, Spatial, StageSpace},
};
use stardust_xr_fusion::{
	fields::Shape,
	spatial::Transform,
	types::{Posef, Resource},
};
use std::{collections::HashMap, fs::DirEntry, path::PathBuf, sync::OnceLock};
use uuid::Uuid;
use xdg::BaseDirectories;

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

#[derive(Default, Serialize, Deserialize)]
pub struct State {
	offset: Posef,
	path: PathBuf,
	#[serde(skip)]
	env: OnceLock<Environment>,
}
impl Migrate for State {
	type Old = Self;
}
impl ClientState for State {
	const APP_ID: &'static str = "org.stardustxr.Atmosphere";

	fn initial_state_update(&mut self) {
		if let Commands::Show { env_name } = Cli::parse().command {
			let config: Config = confy::load("atmosphere", "atmosphere").unwrap();

			self.path = if let Some(env_name) = env_name {
				valid_environments()
					.get(&env_name)
					.map(DirEntry::path)
					.unwrap_or(config.environment)
			} else {
				config.environment
			};
		} else {
			println!("somehow ran initial_state_update without using the show command")
		}
	}
}
impl Reify for State {
	fn reify(
		&self,
		_context: &Context,
		_tasks: impl Tasker<Self>,
	) -> impl stardust_xr_asteroids::Element<Self> {
		let env = self
			.env
			.get_or_init(|| Environment::load(self.path.join("env.kdl"), &self.path));
		let sky_light = env.sky_light.clone().map(|v| {
			SkyLight(Resource::Direct {
				path: v.to_string_lossy().to_string(),
			})
			.build()
		});
		let sky_tex = env.sky_tex.clone().map(|v| {
			SkyTex {
				resource: Resource::Direct {
					path: v.to_string_lossy().to_string(),
				},
				opaque: true,
			}
			.build()
		});
		StageSpace.build().child(
			Entity::new(Shape::Sphere { radius: 1000000.0 })
				.pose(self.offset)
				.component(Poseable::new(|state: &mut Self, pose| {
					state.offset = pose;
				}))
				.component(stardust_xr_asteroids::components::Environment)
				.component(Container)
				.build()
				.maybe_child(sky_light)
				.maybe_child(sky_tex)
				.child(reify_node(&env.root).1),
		)
	}
}

fn reify_node(node: &Node) -> (Uuid, DynamicElement<State>) {
	let node_type = &node.node_type;
	let children = node.children.iter().map(reify_node);
	(
		node.uuid,
		match node_type {
			NodeType::Spatial => Spatial(node.transform)
				.build()
				.stable_children(children)
				.dynamic(),

			NodeType::Model(path_buf) => match Model::direct(path_buf) {
				Err(err) => {
					println!(
						"Error while loading model: {err}, from: {}",
						path_buf.to_string_lossy()
					);
					Spatial(node.transform)
						.build()
						.stable_children(children)
						.dynamic()
				}
				Ok(v) => v
					.transform(node.transform)
					.build()
					.stable_children(children)
					.dynamic(),
			},

			NodeType::Box(scale) => Spatial({
				let scale = Vec3::from(node.transform.scale) * *scale;
				Transform {
					scale: scale.into(),
					..node.transform
				}
			})
			.build()
			.stable_children(children)
			.dynamic(),
		},
	)
}

#[tokio::main(flavor = "current_thread")]
async fn show() {
	stardust_xr_asteroids::client::run::<State>(&[])
		.await
		.unwrap();
}

#[inline]
pub fn environment_dirs() -> Vec<PathBuf> {
	let basedirs = BaseDirectories::with_prefix("xr_environments");
	let mut data_dirs = basedirs.get_data_dirs();
	if let Some(data_home) = basedirs.get_data_home() {
		data_dirs.push(data_home);
	}
	data_dirs
}

pub fn valid_environments() -> HashMap<String, DirEntry> {
	environment_dirs()
		.into_iter()
		.rev()
		.filter_map(|d| d.read_dir().ok())
		.flatten()
		.filter_map(|env| env.ok())
		.filter(|env| !env.file_type().unwrap().is_file())
		.map(|env| (env.file_name().to_string_lossy().to_string(), env))
		.collect()
}
