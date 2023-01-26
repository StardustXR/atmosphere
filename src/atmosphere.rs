use crate::{environment::Environment, environment_data::EnvironmentData};
use color_eyre::eyre::Result;
use mint::Vector3;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{client::Client, core::values::Transform, spatial::Spatial};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	height: f32,
	environment: PathBuf,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			height: 1.65,
			environment: PathBuf::default(),
		}
	}
}

#[allow(dead_code)]
pub struct Atmosphere {
	config: Config,
	environment: Option<Environment>,
	root: Spatial,
}
impl Atmosphere {
	pub fn new(client: &Arc<Client>) -> Result<Self> {
		let config: Config = dbg!(confy::load("atmosphere", "atmosphere")?);
		let data_path = dirs::config_dir()
			.unwrap()
			.join("atmosphere/environments")
			.join(&config.environment)
			.join("env.toml");
		let root = Spatial::create(client.get_root(), Transform::default(), false)?;
		root.set_position(Some(client.get_hmd()), Vector3::from([0.0, 0.0, 0.0]))?;
		root.set_position(None, Vector3::from([0.0, -config.height, 0.0]))?;
		let environment_data = EnvironmentData::load(&data_path)?;
		let environment = Some(Environment::from_data(&root, data_path, environment_data)?);
		dbg!(&environment);
		Ok(Atmosphere {
			config,
			root,
			environment,
		})
	}
}
