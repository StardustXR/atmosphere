mod atmosphere;
mod auto_zone_capture;
mod config;
mod environment;
mod environment_data;
mod play_space;

use crate::config::Config;
use atmosphere::Atmosphere;
use clap::{Parser, Subcommand};
use copy_dir::copy_dir;
use stardust_xr_fusion::{client::Client, node::NodeType, root::RootAspect};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
	Install { path: PathBuf },
	SetDefault { env_name: String },
	Show { env_name: Option<String> },
}

fn main() {
	let args = Cli::parse();
	let config: Config = confy::load("atmosphere", "atmosphere").unwrap();
	match args.command {
		Commands::Install { path } => install(path),
		Commands::SetDefault { env_name } => set_default(config, env_name),
		Commands::Show { env_name } => show(&config, env_name),
	}
}

#[tokio::main(flavor = "current_thread")]
async fn show(config: &Config, env_name: Option<String>) {
	let (client, event_loop) = Client::connect_with_async_loop()
		.await
		.expect("Connect to stardust server failed");
	let _atmosphere = client
		.get_root()
		.alias()
		.wrap(Atmosphere::new(&client, &config, env_name).await)
		.unwrap();

	tokio::select! {
		e = tokio::signal::ctrl_c() => {
			_atmosphere.lock_wrapped().reset();
			e.unwrap()
		},
		e = event_loop => e.unwrap().unwrap(),
	}
}

fn install(path: PathBuf) {
	let environment_dir = dirs::config_dir().unwrap().join("atmosphere/environments");
	if std::fs::metadata(path.join("env.toml")).is_err() {
		panic!("{} does not contain an env.toml file!", path.display());
	}
	let dest_path = environment_dir.join(path.file_name().unwrap());
	copy_dir(path, &dest_path).unwrap();
	println!(
		"Installed environment {} to {}",
		dest_path.file_name().unwrap().to_string_lossy(),
		dest_path.display()
	);
}

fn set_default(mut config: Config, env_name: String) {
	let environment_dir = dirs::config_dir()
		.unwrap()
		.join("atmosphere/environments")
		.join(&env_name);
	if std::fs::metadata(environment_dir).is_err() {
		panic!("Environment {env_name} does not exist, you may have to install it.");
	}

	config.environment = env_name.into();

	confy::store("atmosphere", "atmosphere", config).unwrap();
}
