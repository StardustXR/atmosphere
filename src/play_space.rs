use mint::Vector2;
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	client::Client,
	core::values::Datamap,
	data::{PulseReceiver, PulseSender, PulseSenderAspect, PulseSenderHandler},
	fields::Field,
	node::{NodeError, NodeType},
	spatial::Transform,
	HandlerWrapper,
};

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

pub struct PlaySpaceFinder(Option<(u64, PulseReceiver)>);
impl PlaySpaceFinder {
	pub fn new(client: &Client) -> Result<HandlerWrapper<PulseSender, PlaySpaceFinder>, NodeError> {
		PulseSender::create(
			client.get_root(),
			Transform::none(),
			&Datamap::from_typed(PlaySpaceMap::default()).unwrap(),
		)?
		.wrap(PlaySpaceFinder(None))
	}
	pub fn play_space(&self) -> Option<PulseReceiver> {
		self.0.as_ref().map(|s| &s.1).map(NodeType::alias)
	}
}
impl PulseSenderHandler for PlaySpaceFinder {
	fn new_receiver(&mut self, receiver: PulseReceiver, _field: Field) {
		self.0
			.replace((receiver.node().get_id().unwrap(), receiver));
	}

	fn drop_receiver(&mut self, id: u64) {
		let Some((self_uid, _rx)) = &self.0 else {
			return;
		};
		if &id == self_uid {
			self.0.take();
		}
	}
}
