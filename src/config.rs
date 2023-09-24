use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub height: f32,
	pub environment: PathBuf,
}
impl Default for Config {
	fn default() -> Self {
		Self {
			height: 1.65,
			environment: PathBuf::default(),
		}
	}
}
