use glam::{EulerRot, Quat, Vec3};
use kdl::{KdlDocument, KdlNode};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct NodeInfo {
	pub translation: Option<Vec3>,
	pub rotation: Option<Quat>,
	pub scale: Option<Vec3>,
	pub children: Vec<Node>,
}
impl NodeInfo {
	fn parse(node: &KdlNode) -> Self {
		let translation = node
			.get("translation")
			.map(|v| Self::parse_vec3(v.value().as_string().unwrap()));

		let rotation = node
			.get("rotation")
			.map(|v| Self::parse_rotation(v.value()));

		let scale = node.get("scale").map(|v| Self::parse_scale(v.value()));

		let children = node
			.children()
			.map(|c| c.nodes().iter().map(parse_node).collect::<Vec<_>>())
			.unwrap_or_default();

		NodeInfo {
			translation,
			rotation,
			scale,
			children,
		}
	}

	fn parse_vec3(s: &str) -> Vec3 {
		let parts: Vec<f32> = s
			.split_whitespace()
			.map(|p| p.parse::<f32>().expect("Failed to parse Vec3 component"))
			.collect();

		if parts.len() != 3 {
			panic!("Invalid Vec3 format");
		}

		Vec3::new(parts[0], parts[1], parts[2])
	}

	fn parse_rotation(prop: &kdl::KdlValue) -> Quat {
		match prop {
			kdl::KdlValue::String(s) => {
				let parts: Vec<f32> = s
					.split_whitespace()
					.map(|p| {
						p.parse::<f32>()
							.expect("Failed to parse rotation component")
					})
					.collect();

				match parts.len() {
					3 => Quat::from_euler(EulerRot::XYZ, parts[0], parts[1], parts[2]),
					4 => Quat::from_xyzw(parts[0], parts[1], parts[2], parts[3]),
					_ => panic!("Invalid rotation format"),
				}
			}
			kdl::KdlValue::Base10Float(n) => Quat::from_rotation_y(*n as f32),
			kdl::KdlValue::Base10(n) => Quat::from_rotation_y(*n as f32),
			_ => panic!("Invalid rotation property type"),
		}
	}

	fn parse_scale(prop: &kdl::KdlValue) -> Vec3 {
		match prop {
			kdl::KdlValue::String(s) => {
				let parts: Vec<f32> = s
					.split_whitespace()
					.map(|p| p.parse::<f32>().expect("Failed to parse scale component"))
					.collect();

				match parts.len() {
					3 => Vec3::new(parts[0], parts[1], parts[2]),
					_ => panic!("Invalid scale format"),
				}
			}
			kdl::KdlValue::Base10Float(n) => Vec3::splat(*n as f32),
			kdl::KdlValue::Base10(n) => Vec3::splat(*n as f32),
			_ => panic!("Invalid scale property type"),
		}
	}
}

#[derive(Debug, Clone)]
pub enum Node {
	Spatial(NodeInfo),
	Model { path: PathBuf, info: NodeInfo },
	Box { size: Vec3, info: NodeInfo },
}

#[derive(Debug, Clone)]
pub struct EnvironmentData {
	pub sky_tex: Option<PathBuf>,
	pub sky_light: Option<PathBuf>,
	pub root: Node,
}

impl EnvironmentData {
	pub fn load(file: impl AsRef<Path>) -> Self {
		let content = std::fs::read_to_string(file).unwrap();
		let doc: KdlDocument = content.parse().unwrap();

		let mut env = EnvironmentData {
			sky_tex: None,
			sky_light: None,
			root: Node::Spatial(NodeInfo {
				translation: None,
				rotation: None,
				scale: None,
				children: Vec::new(),
			}),
		};

		for node in doc.nodes() {
			match node.name().value() {
				"sky" => {
					let path = node
						.get("path")
						.map(|v| v.value().as_string().unwrap().into());
					env.sky_tex.clone_from(&path);
					env.sky_light = path;
				}
				"sky_tex" => {
					env.sky_tex = node
						.get("path")
						.map(|v| v.value().as_string().unwrap().into())
				}
				"sky_light" => {
					env.sky_light = node
						.get("path")
						.map(|v| v.value().as_string().unwrap().into())
				}
				"root" => env.root = parse_node(node),
				_ => panic!("Unknown node type: {}", node.name().value()),
			}
		}

		env
	}
}

fn parse_node(node: &KdlNode) -> Node {
	let info = NodeInfo::parse(node);

	match node.name().value() {
		"spatial" | "root" => Node::Spatial(info),
		"model" => {
			let path = node
				.get("path")
				.expect("Model node missing path")
				.value()
				.as_string()
				.expect("Model node path attribute is not string")
				.into();
			Node::Model { path, info }
		}
		"box" => {
			let size = node
				.get("size")
				.expect("Box node missing size")
				.value()
				.as_string()
				.map(NodeInfo::parse_vec3)
				.expect("Invalid box size");
			Node::Box { size, info }
		}
		_ => panic!("Unknown node type: {}", node.name().value()),
	}
}
