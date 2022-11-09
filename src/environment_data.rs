use anyhow::Result;
use glam::{EulerRot, Quat};
use mint::{Quaternion, Vector3};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::{
	fs::read_to_string,
	path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Rotation {
	Yaw(f32),
	Euler(Vector3<f32>),
	Quat(Quaternion<f32>),
}
impl Rotation {
	pub fn to_quat(&self) -> Quaternion<f32> {
		match self {
			Rotation::Yaw(yaw) => Quat::from_rotation_y(*yaw).into(),
			Rotation::Euler(euler) => {
				Quat::from_euler(EulerRot::XYZ, euler.x, euler.y, euler.z).into()
			}
			Rotation::Quat(quat) => *quat,
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct SpatialData {
	pub parent: Option<String>,
	pub position: Option<Vector3<f32>>,
	pub rotation: Option<Rotation>,
	pub scale: Option<Vector3<f32>>,
}

#[derive(Debug, Deserialize)]
pub struct ModelData {
	pub path: PathBuf,
	pub spatial: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BoxFieldData {
	pub spatial: String,
	pub size: Vector3<f32>,
}

#[derive(Debug, Deserialize)]
pub struct EnvironmentData {
	pub sky: Option<PathBuf>,
	pub sky_tex: Option<PathBuf>,
	pub sky_light: Option<PathBuf>,
	pub root: Vector3<f32>,
	pub spatials: FxHashMap<String, SpatialData>,
	pub models: FxHashMap<String, ModelData>,
	pub box_fields: FxHashMap<String, BoxFieldData>,
}
impl EnvironmentData {
	pub fn load(file: impl AsRef<Path>) -> Result<Self> {
		toml::from_str(&read_to_string(file)?).map_err(|err| err.into())
	}
}
