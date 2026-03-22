use crate::{config::Config, valid_environments};
use copy_dir::copy_dir;
use std::path::PathBuf;

pub fn list() {
	let default_name = confy::load::<Config>("atmosphere", "atmosphere")
		.ok()
		.and_then(|config| {
			config
				.environment
				.file_name()
				.map(|file_name| file_name.to_string_lossy().to_string())
		});

	for (name, dir) in valid_environments() {
		let is_default = default_name
			.as_ref()
			.is_some_and(|default| default == &name);
		let status = dir.path().join("env.kdl").exists();
		match (is_default, status) {
			(true, true) => {
				println!(
					"{}",
					ansi_term::Color::Blue
						.bold()
						.paint(format!("> {name}: valid, default"))
				);
			}
			(true, false) => {
				println!(
					"{}",
					ansi_term::Color::Blue
						.bold()
						.strikethrough()
						.paint(format!("> {name}: invalid (missing env.kdl), default"))
				);
			}
			(false, true) => {
				println!("  {name}: valid");
			}
			(false, false) => {
				println!("  {name}: invalid (missing env.kdl)");
			}
		}
		println!(
			"    └ {}",
			ansi_term::Color::Black
				.dimmed()
				.paint(dir.path().to_string_lossy())
		);
	}
}

pub fn install(path: PathBuf) {
	let Some(environment_dir) = dirs::data_local_dir() else {
		panic!("Could not find a suitable data directory for environments.");
	};
	if !path.join("env.kdl").exists() {
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
	let valid_environments = valid_environments();
	let Some(default_env) = valid_environments.get(&env_name) else {
		panic!("Environment {env_name} does not exist, you may have to install it.");
	};

	config.environment = default_env.path();
	confy::store("atmosphere", "atmosphere", config).unwrap();
	println!(
		"Set environment {} to default at path {}",
		default_env.file_name().to_string_lossy(),
		default_env.path().display()
	);
}
