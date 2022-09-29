use crate::environment_data::{EnvironmentData, Rotation};
use anyhow::Result;
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	drawable::Model,
	node::{NodeError, NodeType},
	spatial::Spatial,
};
use std::{fmt::Debug, path::PathBuf};

pub struct Environment {
	data: EnvironmentData,
	root: Spatial,
	models: FxHashMap<String, Model>,
}
impl Environment {
	pub fn from_data(
		parent: &Spatial,
		config_path: PathBuf,
		data: EnvironmentData,
	) -> Result<Self> {
		let root = Spatial::builder()
			.position(data.root)
			.spatial_parent(parent)
			.zoneable(false)
			.build()?;
		let client = parent.client().unwrap();
		let config_folder = config_path.parent().unwrap();
		if let Some(sky) = &data.sky {
			client.set_sky_tex_light(&config_folder.join(sky)).unwrap();
		}
		if let Some(sky_tex) = &data.sky_tex {
			client.set_sky_tex(&config_folder.join(sky_tex)).unwrap();
		}
		if let Some(sky_light) = &data.sky_light {
			client
				.set_sky_light(&config_folder.join(sky_light))
				.unwrap();
		}
		let models: Result<FxHashMap<String, Model>, NodeError> = data
			.models
			.iter()
			.map(|(name, data)| {
				let path = config_folder.join(&data.path);
				let model = Model::builder()
					.resource(&path)
					.spatial_parent(&root)
					.and_position(data.position)
					.and_rotation(data.rotation.as_ref().map(Rotation::to_quat))
					.build();

				Ok((name.clone(), model?))
			})
			.collect();
		let models = models?;
		for (name, model) in models.iter() {
			if let Some(data) = data.models.get(name) {
				if let Some(parent) = data.parent.as_ref().and_then(|parent| models.get(parent)) {
					model.set_spatial_parent(parent).unwrap();
				}
			}
		}

		Ok(Environment { data, root, models })
	}
}
impl Debug for Environment {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Environment")
			.field("data", &self.data)
			.field("root", &self.root.node().get_path())
			.field("models", &self.models.keys())
			.finish()
	}
}
