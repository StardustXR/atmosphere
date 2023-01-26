pub mod atmosphere;
pub mod environment;
pub mod environment_data;

use atmosphere::Atmosphere;
use color_eyre::eyre::Result;
use stardust_xr_fusion::client::Client;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
	let (client, event_loop) = Client::connect_with_async_loop().await?;
	// client.set_base_prefixes(&[directory_relative_path!("res")]);

	let _atmosphere = Atmosphere::new(&client)?;

	tokio::select! {
		e = tokio::signal::ctrl_c() => e?,
		e = event_loop => e??,
	};
	Ok(())
}
