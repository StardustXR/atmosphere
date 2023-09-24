use crate::{environment::Environment, environment_data::EnvironmentData, Config};
use mint::Vector3;
use stardust_xr_fusion::{client::Client, core::values::Transform, spatial::Spatial};
use std::{path::Path, sync::Arc};

#[allow(dead_code)]
pub struct Atmosphere {
	environment: Environment,
	root: Spatial,
}
impl Atmosphere {
	pub fn new(client: &Arc<Client>, config: &Config, env_name: Option<String>) -> Self {
		let data_path = dirs::config_dir()
			.unwrap()
			.join("atmosphere/environments")
			.join(
				env_name
					.as_ref()
					.map(Path::new)
					.unwrap_or(&config.environment),
			)
			.join("env.toml");
		let root = Spatial::create(client.get_root(), Transform::default(), false).unwrap();
		root.set_position(Some(client.get_hmd()), Vector3::from([0.0, 0.0, 0.0]))
			.unwrap();
		root.set_position(None, Vector3::from([0.0, -config.height, 0.0]))
			.unwrap();
		let environment_data = EnvironmentData::load(&data_path).unwrap();
		let environment = Environment::from_data(&root, data_path, environment_data).unwrap();
		dbg!(&environment);
		Atmosphere { root, environment }
	}
}
