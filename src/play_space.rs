use mint::Vector2;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	client::Client,
	core::values::Transform,
	data::{NewReceiverInfo, PulseReceiver, PulseSender, PulseSenderHandler},
	fields::UnknownField,
	node::{NodeError, NodeType},
	spatial::Spatial,
	HandlerWrapper,
};
use stardust_xr_molecules::datamap::Datamap;

#[derive(Debug, Deserialize, Serialize)]
struct PlaySpaceMap {
	play_space: (),
	size: Vector2<f32>,
}
impl Default for PlaySpaceMap {
	fn default() -> Self {
		Self {
			play_space: (),
			size: [0.0; 2].into(),
		}
	}
}

pub struct PlaySpaceFinder(Option<PulseReceiver>);
impl PlaySpaceFinder {
	pub fn new(client: &Client) -> Result<HandlerWrapper<PulseSender, PlaySpaceFinder>, NodeError> {
		PulseSender::create(
			client.get_root(),
			Transform::none(),
			&Datamap::create(PlaySpaceMap::default()).serialize(),
		)?
		.wrap(PlaySpaceFinder(None))
	}
	pub fn play_space(&self) -> Option<Spatial> {
		self.0.as_deref().map(NodeType::alias)
	}
}
impl PulseSenderHandler for PlaySpaceFinder {
	fn new_receiver(
		&mut self,
		_info: NewReceiverInfo,
		receiver: PulseReceiver,
		_field: UnknownField,
	) {
		self.0.replace(receiver);
	}

	fn drop_receiver(&mut self, uid: &str) {
		let Some(rx) = &self.0 else {return};
		if rx.node().get_name().unwrap() == uid {
			self.0.take();
		}
	}
}
