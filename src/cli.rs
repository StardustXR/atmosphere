use crate::{config::Config, environments_dir, get_list};
use copy_dir::copy_dir;
use std::path::PathBuf;

pub fn list() {
	for dir in get_list() {
		let status = dir.path().join("env.kdl").exists();
		println!(
			"{}: {}",
			dir.file_name().to_string_lossy(),
			if status {
				"valid"
			} else {
				"invalid (missing env.kdl)"
			}
		);
	}
}
pub fn install(path: PathBuf) {
	let environment_dir = environments_dir();
	if std::fs::metadata(path.join("env.kdl")).is_err() {
		panic!("{} does not contain an env.kdl file!", path.display());
	}
	let dest_path = environment_dir.join(path.file_name().unwrap());
	copy_dir(path, &dest_path).unwrap();
	println!(
		"Installed environment {} to {}",
		dest_path.file_name().unwrap().to_string_lossy(),
		dest_path.display()
	);
}

pub fn set_default(mut config: Config, env_name: String) {
	let environment_dir = environments_dir().join(&env_name);
	if std::fs::metadata(environment_dir).is_err() {
		panic!("Environment {env_name} does not exist, you may have to install it.");
	}

	config.environment = env_name.into();

	confy::store("atmosphere", "atmosphere", config).unwrap();
}
