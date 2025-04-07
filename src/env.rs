use std::path::{Path, PathBuf};

use glam::{EulerRot, Quat, Vec3};
use kdl::{KdlDocument, KdlNode};
use stardust_xr_fusion::spatial::Transform;
use uuid::Uuid;

pub enum NodeType {
	Spatial,
	Model(PathBuf),
	Box(Vec3),
}
pub struct Node {
	pub uuid: Uuid,
	pub transform: Transform,
	pub children: Vec<Node>,
	pub node_type: NodeType,
}
pub struct Environment {
	pub sky_tex: Option<PathBuf>,
	pub sky_light: Option<PathBuf>,
	pub root: Node,
}
impl Environment {
	pub fn load(file: impl AsRef<Path>, env_path: impl AsRef<Path>) -> Self {
		let content = std::fs::read_to_string(file.as_ref()).unwrap();
		let doc: KdlDocument = content.parse().unwrap();

		let mut env = Environment {
			sky_tex: None,
			sky_light: None,
			root: Node {
				uuid: Uuid::new_v4(),
				transform: Transform::none(),
				children: Vec::new(),
				node_type: NodeType::Spatial,
			},
		};

		for node in doc.nodes() {
			match node.name().value() {
				"sky" => {
					let path = node
						.get("path")
						.map(|v| v.value().as_string().unwrap().into())
						.map(|v: PathBuf| env_path.as_ref().join(v));
					env.sky_tex.clone_from(&path);
					env.sky_light = path;
				}
				"sky_tex" => {
					env.sky_tex = node
						.get("path")
						.map(|v| v.value().as_string().unwrap().into())
						.map(|v: PathBuf| env_path.as_ref().join(v))
				}
				"sky_light" => {
					env.sky_light = node
						.get("path")
						.map(|v| v.value().as_string().unwrap().into())
						.map(|v: PathBuf| env_path.as_ref().join(v))
				}
				"root" => env.root = parse_node(node, env_path.as_ref()),
				_ => panic!("Unknown node type: {}", node.name().value()),
			}
		}

		env
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
fn parse_node(node: &KdlNode, env_path: &Path) -> Node {
	let translation = node
		.get("translation")
		.map(|v| parse_vec3(v.value().as_string().unwrap()));

	let rotation = node.get("rotation").map(|v| parse_rotation(v.value()));

	let scale = node.get("scale").map(|v| parse_scale(v.value()));

	let children = node
		.children()
		.map(|c| {
			c.nodes()
				.iter()
				.map(|node| parse_node(node, env_path))
				.collect::<Vec<_>>()
		})
		.unwrap_or_default();

	let node_type = match node.name().value() {
		"spatial" | "root" => NodeType::Spatial,
		"model" => {
			let path = node
				.get("path")
				.expect("Model node missing path")
				.value()
				.as_string()
				.expect("Model node path attribute is not string");
			NodeType::Model(env_path.join(path))
		}
		"box" => {
			let size = node
				.get("size")
				.expect("Box node missing size")
				.value()
				.as_string()
				.map(parse_vec3)
				.expect("Invalid box size");
			NodeType::Box(size)
		}
		_ => panic!("Unknown node type: {}", node.name().value()),
	};

	Node {
		uuid: Uuid::new_v4(),
		transform: Transform {
			translation: translation.map(Into::into),
			rotation: rotation.map(Into::into),
			scale: scale.map(Into::into),
		},
		children,
		node_type,
	}
}
