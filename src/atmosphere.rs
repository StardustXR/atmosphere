use crate::{
	auto_zone_capture::AutoZoneCapture, environment::Environment,
	environment_data::EnvironmentData, play_space::PlaySpaceFinder, Config,
};
use glam::{vec3, Mat4, Vec3};
use stardust_xr_fusion::{
	client::{Client, ClientState, FrameInfo, RootHandler},
	core::values::rgba_linear,
	data::PulseSender,
	drawable::{Lines, LinesAspect},
	fields::SphereField,
	input::{InputDataType, InputHandler},
	node::NodeType,
	spatial::{Spatial, SpatialAspect, Transform, Zone, ZoneAspect},
	HandlerWrapper,
};
use stardust_xr_molecules::{
	input_action::{BaseInputAction, InputActionHandler, SingleActorAction},
	lines::{circle, LineExt},
};
use std::{f32::consts::FRAC_PI_2, path::Path, sync::Arc};

#[allow(dead_code)]
pub struct Atmosphere {
	play_space_finder: HandlerWrapper<PulseSender, PlaySpaceFinder>,
	reference_space: Spatial,
	root: Spatial,
	environment: Environment,

	_zone_field: SphereField,
	zone: HandlerWrapper<Zone, AutoZoneCapture>,

	offset: Vec3,
	velocity: Vec3,
	signifiers: Lines,
	_input_field: SphereField,
	input_handler: HandlerWrapper<InputHandler, InputActionHandler<()>>,
	previous_position: Option<Vec3>,
	condition_action: BaseInputAction<()>,
	move_action: SingleActorAction<()>,
}
impl Atmosphere {
	pub fn new(client: &Arc<Client>, config: &Config, env_name: Option<String>) -> Self {
		let data_path = dirs::config_dir()
			.unwrap()
			.join("atmosphere/environments")
			.join(
				env_name
					.as_ref()
					.map(Path::new)
					.unwrap_or(&config.environment),
			)
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

		let _zone_field = SphereField::create(&root, [0.0; 3], 1000.0).unwrap();
		let zone = Zone::create(&root, Transform::identity(), &_zone_field).unwrap();
		let zone_handler = AutoZoneCapture(zone.alias(), Default::default());
		let zone = zone.wrap(zone_handler).unwrap();

		let _input_field = SphereField::create(client.get_hmd(), [0.0; 3], 0.0).unwrap();
		let input_handler = InputActionHandler::wrap(
			InputHandler::create(client.get_root(), Transform::identity(), &_input_field).unwrap(),
			(),
		)
		.unwrap();
		let condition_action = BaseInputAction::new(false, |d, _| d.order == 0);
		let move_action = SingleActorAction::new(
			true,
			|data, _| {
				data.datamap.with_data(|d| match &data.input {
					InputDataType::Hand(_) => d.idx("grab_strength").as_f32() > 0.9,
					_ => d.idx("grab").as_f32() > 0.9,
				})
			},
			true,
		);
		let signifiers =
			Lines::create(input_handler.node().as_ref(), Transform::identity(), &[]).unwrap();

		Atmosphere {
			reference_space,
			root,
			environment,

			_zone_field,
			zone,

			offset: Default::default(),
			velocity: Default::default(),
			_input_field,
			input_handler,
			previous_position: None,
			condition_action,
			move_action,
			signifiers,

			play_space_finder,
		}
	}

	fn input_update(&mut self, info: FrameInfo) {
		self.input_handler
			.lock_wrapped()
			.update_actions([&mut self.condition_action, self.move_action.base_mut()]);
		self.move_action.update(Some(&mut self.condition_action));
		// dbg!(&self.condition_action.currently_acting.len());
		// dbg!(&self.move_action.actor().is_some());

		// draw the lines to indicate we can move the world
		let signifier_lines = self
			.condition_action
			.currently_acting
			.union(&self.move_action.base().currently_acting)
			.filter_map(|i| match &i.input {
				InputDataType::Hand(h) => Some((
					i,
					Mat4::from_rotation_translation(h.palm.rotation.into(), h.palm.position.into()),
				)),
				InputDataType::Tip(t) => Some((
					i,
					Mat4::from_rotation_translation(t.orientation.into(), t.origin.into()),
				)),
				_ => None,
			})
			.map(|(i, t)| {
				let line = circle(64, 0.0, 0.1)
					.transform(Mat4::from_rotation_x(FRAC_PI_2))
					.transform(Mat4::from_translation(vec3(0.0, 0.05, -0.02)))
					.transform(t);

				let grabbed = self.move_action.actor() == Some(i);

				if grabbed {
					line.color(rgba_linear!(0.0, 0.549, 1.0, 1.0))
				} else {
					line
				}
			})
			.collect::<Vec<_>>();
		self.signifiers.set_lines(&signifier_lines).unwrap();

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
				dbg!(offset_magnify);
				self.velocity += offset.normalize_or_zero() * offset_magnify;
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

	pub fn reset(&mut self) {
		println!("Reset");
		self.root
			.set_local_transform(Transform::identity())
			.unwrap();
	}
}

impl RootHandler for Atmosphere {
	fn frame(&mut self, info: FrameInfo) {
		self.velocity *= 0.99;
		self.input_update(info);
		self.offset += self.velocity;
		self.offset.y = self.offset.y.min(0.0);
		dbg!(self.velocity);
		let _ = self
			.root
			.set_local_transform(Transform::from_translation(self.offset));

		self.zone.node().update().unwrap();
		if let Some(play_space) = self.play_space_finder.lock_wrapped().play_space().take() {
			let _ = self
				.reference_space
				.set_relative_transform(&play_space, Transform::identity());
		}
	}

	fn save_state(&mut self) -> ClientState {
		ClientState::default()
	}
}
