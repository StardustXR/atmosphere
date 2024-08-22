use crate::environment_data::*;
use color_eyre::eyre::Result;
use stardust_xr_fusion::{
	core::values::ResourceID,
	drawable::{set_sky_light, set_sky_tex, Model},
	node::NodeType,
	spatial::{Spatial, SpatialRefAspect, Transform},
};
use std::{
	fmt::Debug,
	path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Environment {
	_data: EnvironmentData,
	_root: Spatial,
	spatials: Vec<Spatial>,
	models: Vec<Model>,
}
impl Environment {
	pub fn from_data(
		parent: &Spatial,
		config_path: PathBuf,
		data: EnvironmentData,
	) -> Result<Self> {
		let client = parent.client().unwrap();
		let config_folder = config_path.parent().unwrap();

		if let Some(sky_tex) = &data.sky_tex {
			let resource = ResourceID::Direct(config_folder.join(sky_tex));
			set_sky_tex(&client, &resource)?;
		}
		if let Some(sky_light) = &data.sky_light {
			let resource = ResourceID::Direct(config_folder.join(sky_light));
			set_sky_light(&client, &resource)?;
		}

		let root = Spatial::create(parent, Transform::identity(), false)?;
		let root2 = root.alias();
		let mut environment = Environment {
			_data: data.clone(),
			_root: root,
			spatials: Default::default(),
			models: Default::default(),
		};
		environment.create_node(&data.root, &root2, config_folder)?;

		Ok(environment)
	}
	fn create_node(
		&mut self,
		node: &Node,
		parent: &impl SpatialRefAspect,
		config_folder: &Path,
	) -> Result<()> {
		let NodeInfo {
			translation,
			rotation,
			scale,
			children,
		} = match node {
			Node::Spatial(info) => info,
			Node::Model { path: _, info } => info,
			Node::Box { size: _, info } => info,
		};
		let transform = Transform {
			translation: translation.map(Into::into),
			rotation: rotation.map(Into::into),
			scale: scale.map(Into::into),
		};

		match node {
			Node::Spatial { .. } => {
				let spatial = Spatial::create(parent, transform, false)?;
				for child in children {
					self.create_node(child, &spatial, config_folder)?;
				}
				self.spatials.push(spatial);
			}
			Node::Model { path, .. } => {
				let model_path = config_folder.join(path);
				let model = Model::create(parent, transform, &ResourceID::Direct(model_path))?;
				for child in children {
					self.create_node(child, &model, config_folder)?;
				}
				self.models.push(model);
			}
			Node::Box { size: _, .. } => {
				let spatial = Spatial::create(parent, transform, false)?;
				// Here you might want to create a box model or handle it differently
				for child in children {
					self.create_node(child, &spatial, config_folder)?;
				}
				self.spatials.push(spatial);
			}
		}
		Ok(())
	}
}
