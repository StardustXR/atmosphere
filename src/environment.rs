use crate::environment_data::{EnvironmentData, Rotation, Scale};
use color_eyre::eyre::Result;
use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	core::values::Transform,
	drawable::{set_sky_light, set_sky_tex, set_sky_tex_light, Model, ResourceID},
	node::{NodeError, NodeType},
	spatial::Spatial,
};
use std::{fmt::Debug, path::PathBuf};

pub struct Environment {
	data: EnvironmentData,
	root: Spatial,
	spatials: FxHashMap<String, Spatial>,
	models: FxHashMap<String, Model>,
}
impl Environment {
	pub fn from_data(
		parent: &Spatial,
		config_path: PathBuf,
		data: EnvironmentData,
	) -> Result<Self> {
		let root = Spatial::create(parent, Transform::from_position(data.root), false)?;
		let client = parent.client().unwrap();
		let config_folder = config_path.parent().unwrap();
		if let Some(sky) = &data.sky {
			set_sky_tex_light(&client, &config_folder.join(sky))?;
		}
		if let Some(sky_tex) = &data.sky_tex {
			set_sky_tex(&client, &config_folder.join(sky_tex))?;
		}
		if let Some(sky_light) = &data.sky_light {
			set_sky_light(&client, &config_folder.join(sky_light))?;
		}
		let spatials_data = data.spatials.clone().unwrap_or_default();
		let spatials: Result<FxHashMap<String, Spatial>, NodeError> = spatials_data
			.iter()
			.map(|(name, data)| {
				let spatial = Spatial::create(
					&root,
					Transform {
						position: data.position,
						rotation: data.rotation.as_ref().map(Rotation::to_quat),
						scale: data.scale.as_ref().map(Scale::to_vec),
					},
					false,
				)?;

				Ok((name.clone(), spatial))
			})
			.collect();
		let spatials = spatials?;
		for (name, spatial) in spatials.iter() {
			if let Some(spatial_data) = spatials_data.get(name) {
				if let Some(parent) = spatial_data
					.parent
					.as_ref()
					.and_then(|parent_name| spatials.get(parent_name))
				{
					spatial.set_spatial_parent(parent).unwrap();
				}
			}
		}
		let models_data = data.models.clone().unwrap_or_default();
		let models: Result<FxHashMap<String, Model>, NodeError> = models_data
			.iter()
			.map(|(name, data)| {
				let path = config_folder.join(&data.path);
				let parent = data
					.spatial
					.as_ref()
					.and_then(|spatial| spatials.get(spatial))
					.unwrap_or(&root);
				let model = Model::create(parent, Transform::default(), &ResourceID::Direct(path))?;

				Ok((name.clone(), model))
			})
			.collect();
		let models = models?;

		Ok(Environment {
			data,
			root,
			spatials,
			models,
		})
	}
}
impl Debug for Environment {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Environment")
			.field("data", &self.data)
			.field("root", &self.root.node().get_path())
			.field("spatials", &self.spatials.keys())
			.field("models", &self.models.keys())
			.finish()
	}
}
