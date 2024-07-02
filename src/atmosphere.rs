use crate::{
	auto_zone_capture::AutoZoneCapture, environment::Environment,
	environment_data::EnvironmentData, play_space::PlaySpaceFinder, Config,
};
use color_eyre::eyre::Result;
use glam::{vec3, Mat4, Vec3};
use serde::{Deserialize, Serialize};
use stardust_xr_fusion::{
	client::Client,
	data::PulseSender,
	drawable::{Line, Lines, LinesAspect},
	fields::{Field, Shape},
	input::{InputData, InputDataType, InputHandler},
	node::NodeType,
	objects::hmd,
	root::{ClientState, FrameInfo, RootHandler},
	spatial::{Spatial, SpatialAspect, Transform, Zone, ZoneAspect},
	values::color::rgba_linear,
	HandlerWrapper,
};
use stardust_xr_molecules::{
	input_action::{InputQueue, InputQueueable, SingleAction},
	lines::{circle, LineExt},
};
use std::{
	f32::consts::FRAC_PI_2,
	path::{Path, PathBuf},
	sync::Arc,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AtmosphereState {
	env_path: Option<PathBuf>,
	offset: Vec3,
	velocity: Vec3,
}

#[allow(dead_code)]
pub struct Atmosphere {
	play_space_finder: HandlerWrapper<PulseSender, PlaySpaceFinder>,
	reference_space: Spatial,
	root: Spatial,
	environment: Environment,

	_zone_field: Field,
	zone: HandlerWrapper<Zone, AutoZoneCapture>,

	state: AtmosphereState,
	signifiers: Lines,
	_input_field: Field,
	input: InputQueue,
	previous_position: Option<Vec3>,
	move_action: SingleAction,
}
impl Atmosphere {
	pub async fn new(client: &Arc<Client>, config: &Config, env_name: Option<String>) -> Self {
		let state: AtmosphereState = client.get_state().data().unwrap_or_default();
		let env_path = state
			.env_path
			.as_deref()
			.or_else(|| env_name.as_ref().map(Path::new))
			.unwrap_or(&config.environment);
		let data_path = dirs::config_dir()
			.unwrap()
			.join("atmosphere/environments")
			.join(env_path)
			.join("env.toml");

		let reference_space = Spatial::create(client.get_root(), Transform::none(), false).unwrap();
		reference_space
			.set_relative_transform(client.get_root(), Transform::from_translation([0.0; 3]))
			.unwrap();
		reference_space
			.set_local_transform(Transform::from_translation([0.0, -config.height, 0.0]))
			.unwrap();
		let root = Spatial::create(&reference_space, Transform::none(), false).unwrap();

		let environment_data = EnvironmentData::load(&data_path).unwrap();
		let environment = Environment::from_data(&root, data_path, environment_data).unwrap();
		dbg!(&environment);
		let play_space_finder = PlaySpaceFinder::new(&client).unwrap();

		let _zone_field =
			Field::create(&root, Transform::identity(), Shape::Sphere(1000.0)).unwrap();
		let zone = Zone::create(&root, Transform::identity(), &_zone_field).unwrap();
		let zone_handler = AutoZoneCapture(zone.alias(), Default::default());
		let zone = zone.wrap(zone_handler).unwrap();

		let _input_field = Field::create(
			&hmd(client).await.unwrap(),
			Transform::identity(),
			Shape::Sphere(0.0),
		)
		.unwrap();
		let input = InputHandler::create(client.get_root(), Transform::identity(), &_input_field)
			.unwrap()
			.queue()
			.unwrap();

		let move_action = SingleAction::default();

		let signifiers = Lines::create(input.handler(), Transform::identity(), &[]).unwrap();

		Atmosphere {
			reference_space,
			root,
			environment,

			_zone_field,
			zone,

			state,
			_input_field,
			input,
			previous_position: None,
			move_action,
			signifiers,

			play_space_finder,
		}
	}

	fn input_update(&mut self, info: FrameInfo) {
		self.move_action.update(
			true,
			&self.input,
			// |d| d.order == 0,
			|data| match &data.input {
				InputDataType::Pointer(_) => false,
				_ => true,
			},
			|data| {
				data.datamap.with_data(|d| match &data.input {
					InputDataType::Hand(_) => d.idx("grab_strength").as_f32() > 0.9,
					_ => d.idx("grab").as_f32() > 0.9,
				})
			},
		);

		self.update_signifiers();

		let position = self.move_action.actor().map(|p| match &p.input {
			InputDataType::Hand(h) => h.palm.position.into(),
			InputDataType::Tip(t) => t.origin.into(),
			_ => unreachable!(),
		});

		if self.move_action.actor_changed() {
			self.previous_position = position;
			return;
		}
		if let Some(previous_position) = self.previous_position {
			if let Some(position) = position {
				let offset = position - previous_position;
				let offset_magnify = (offset.length() * info.delta as f32).powf(0.9);
				// dbg!(offset_magnify);
				self.state.velocity += offset.normalize_or_zero() * offset_magnify;
				// let _ = self
				// .root
				// .set_relative_transform(&self.root, Transform::from_translation(offset));
				// let _ = self
				// 	.root
				// 	.set_local_transform(Transform::from_translation(self.offset));
			}
		}

		self.previous_position = position;
	}

	// draw the lines to indicate we can move the world
	fn update_signifiers(&self) {
		let mut signifier_lines = self
			.move_action
			.hovering()
			.current()
			.iter()
			.map(|input| Self::generate_signifier(input, false))
			.collect::<Vec<_>>();
		signifier_lines.extend(
			self.move_action
				.actor()
				.map(|input| Self::generate_signifier(input, true)),
		);
		self.signifiers.set_lines(&signifier_lines).unwrap();
	}

	fn generate_signifier(input: &InputData, grabbing: bool) -> Line {
		let transform = match &input.input {
			InputDataType::Pointer(_) => panic!("awawawawawawa"),
			InputDataType::Hand(h) => {
				Mat4::from_rotation_translation(h.palm.rotation.into(), h.palm.position.into())
					* Mat4::from_translation(vec3(0.0, 0.05, -0.02))
					* Mat4::from_rotation_x(FRAC_PI_2)
			}
			InputDataType::Tip(t) => {
				Mat4::from_rotation_translation(t.orientation.into(), t.origin.into())
			}
		};

		let line = circle(
			64,
			0.0,
			match &input.input {
				InputDataType::Pointer(_) => panic!("awawawawawawa"),
				InputDataType::Hand(_) => 0.1,
				InputDataType::Tip(_) => 0.0025,
			},
		)
		.transform(transform);
		if grabbing {
			line.color(rgba_linear!(0.0, 0.549, 1.0, 1.0))
		} else {
			line
		}
	}

	pub fn reset(&mut self) {
		println!("Reset");
		self.root
			.set_local_transform(Transform::identity())
			.unwrap();
	}
}

impl RootHandler for Atmosphere {
	fn frame(&mut self, info: FrameInfo) {
		self.state.velocity *= 0.99;
		self.input_update(info);
		self.state.offset += self.state.velocity;
		self.state.offset.y = self.state.offset.y.min(0.0);
		// dbg!(self.velocity);
		let _ = self
			.root
			.set_local_transform(Transform::from_translation(self.state.offset));

		self.zone.node().update().unwrap();
		if let Some(play_space) = self.play_space_finder.lock_wrapped().play_space().take() {
			let _ = self
				.reference_space
				.set_relative_transform(&play_space, Transform::identity());
		}
	}

	fn save_state(&mut self) -> Result<ClientState> {
		ClientState::from_data_root(Some(self.state.clone()), &self.reference_space)
	}
}
