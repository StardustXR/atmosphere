pub mod atmosphere;
pub mod environment;
pub mod environment_data;

use atmosphere::Atmosphere;
// use manifest_dir_macros::directory_relative_path;
use stardust_xr_fusion::client::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	let (client, event_loop) = Client::connect_with_async_loop()
		.await
		.expect("Unable to connect to server");
	// client.set_base_prefixes(&[directory_relative_path!("res")]);

	let _root = client.wrap_root(Atmosphere::new(&client));

	tokio::select! {
		e = event_loop => e.unwrap(),
		e = tokio::signal::ctrl_c() => e.unwrap(),
	}
}
