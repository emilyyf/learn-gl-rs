#![allow(deref_nullptr)]

extern crate cgmath;
extern crate gl;
extern crate tobj;

use crate::shader::*;
use crate::image::GenericImage;

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(std::ptr::null() as *const $ty)).$field as *const _ as usize
    }
}

#[derive(Clone)]
struct Vertex {
	position: cgmath::Vector3<f32>,
	normal: cgmath::Vector3<f32>,
	tex_coords: cgmath::Vector2<f32>,
}

#[derive(Clone)]
struct Texture {
	id: u32,
	type_: std::string::String,
	path: std::string::String,
}

#[derive(Default, Clone)]
struct Mesh {
	vertices: std::vec::Vec<Vertex>,
	indices: std::vec::Vec<u32>,
	textures: std::vec::Vec<Texture>,
	vao: u32,
	vbo: u32,
	ebo: u32,
}

#[derive(Default, Clone)]
pub struct Model {
	meshes: std::vec::Vec<Mesh>,
	textures_loaded: std::vec::Vec<Texture>,
	directory: std::string::String,
}

fn draw_mesh(mesh: &Mesh, shader: u32) {
	let mut diffuse_nr: u32 = 1;
	let mut specular_nr: u32 = 1;

	for (i, texture) in mesh.textures.iter().enumerate() {
		unsafe {
			gl::ActiveTexture(gl::TEXTURE0 as u32 + i as u32);
			let number;
			if texture.type_ == "diffuse" {
				number = diffuse_nr;
				diffuse_nr += 1;
			} else if texture.type_ == "specular" {
				number = specular_nr;
				specular_nr += 1;
			} else {
				// shouldn't happen
				number = 0;
			}
			set_int(shader, &format!("material.texture_{}{}", texture.type_, number), i as i32);
			gl::BindTexture(gl::TEXTURE_2D, texture.id);
		}
	}
	unsafe {
		gl::ActiveTexture(gl::TEXTURE0);
		gl::BindVertexArray(mesh.vao);
		gl::DrawElements(gl::TRIANGLES, mesh.indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null());
	}
}

	
	
	
fn setup_mesh(vertices: std::vec::Vec<Vertex>, indices: std::vec::Vec<u32>, textures: std::vec::Vec<Texture>) -> Mesh {
	let mut mesh: Mesh = Mesh::default();
	mesh.vertices = vertices;
	mesh.indices = indices;
	mesh.textures = textures;
	unsafe {
		gl::GenVertexArrays(1, &mut mesh.vao);
		gl::GenBuffers(1, &mut mesh.vbo);
		gl::GenBuffers(1, &mut mesh.ebo);

		gl::BindVertexArray(mesh.vao);

		gl::BindBuffer(gl::ARRAY_BUFFER, mesh.vbo);
		let size = (mesh.vertices.len() *  std::mem::size_of::<Vertex>()) as isize;
		let data = &mesh.vertices[0] as *const Vertex as *const std::os::raw::c_void;
		gl::BufferData(gl::ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

		gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, mesh.ebo);
		let size = (mesh.indices.len() * std::mem::size_of::<u32>()) as isize;
		let data = &mesh.indices[0] as *const u32 as *const std::os::raw::c_void;
		gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size, data, gl::STATIC_DRAW);

		let size = std::mem::size_of::<Vertex>() as i32;
		gl::EnableVertexAttribArray(0);
		gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, position) as *const std::os::raw::c_void);

		gl::EnableVertexAttribArray(1);
		gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, normal) as *const std::os::raw::c_void);

		gl::EnableVertexAttribArray(2);
		gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, size, offset_of!(Vertex, tex_coords) as *const std::os::raw::c_void);

		gl::BindVertexArray(0);
	}
	mesh
}

pub fn draw_model(model: &Model, shader: u32) {
	for mesh in model.meshes.iter() {
		draw_mesh(mesh, shader);
	}
}

pub fn load_model(path: &str) -> Model {
	let obj = tobj::load_obj(path, true);
	let (models, materials) = obj.unwrap();
	let mut loaded_model = Model::default();

	for model in models {
		let mesh = &model.mesh;
		let num_vertices = mesh.positions.len() / 3;

		let mut vertices: std::vec::Vec<Vertex> = std::vec::Vec::with_capacity(num_vertices);
		let indices: std::vec::Vec<u32> = mesh.indices.clone();

		let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);
		for i in 0..num_vertices {
			vertices.push(Vertex {
				position: cgmath::vec3(p[i*3], p[i*3+1], p[i*3+2]),
				normal: cgmath::vec3(n[i*3], n[i*3+1], n[i*3+2]),
				tex_coords: cgmath::vec2(t[i*2], t[i*2+1]),
			})
		}

		let mut textures: std::vec::Vec<Texture> = std::vec::Vec::new();
		if let Some(material_id) = mesh.material_id {
			let material = &materials[material_id];

			if !material.diffuse_texture.is_empty() {
				let texture = load_material_texture(&mut loaded_model, &material.diffuse_texture, "diffuse");
				textures.push(texture);
			}

			if !material.specular_texture.is_empty() {
				let texture = load_material_texture(&mut loaded_model, &material.specular_texture, "specular");
				textures.push(texture);
			}

			if !material.normal_texture.is_empty() {
				let texture = load_material_texture(&mut loaded_model, &material.normal_texture, "normal");
				textures.push(texture);
			}
		}

		loaded_model.meshes.push(setup_mesh(vertices, indices, textures));
	}

	loaded_model
}

fn load_material_texture(model: &mut Model, path: &str, type_name: &str) -> Texture {
	let texture = model.textures_loaded.iter().find(|t| t.path == path);
	if let Some(texture) = texture {
		return texture.clone();
	}
	let texture = Texture {
		id: texture_from_file(path),
		type_: type_name.into(),
		path: path.into(),
	};

	model.textures_loaded.push(texture.clone());

	texture
}

fn texture_from_file(path: &str) -> u32 {
	let mut texture_id = 0;

	let img = image::open(path).expect("Texture failed to load");
	// let img = img.flipv();
	let format = match img {
		image::ImageLuma8(_) => gl::RED,
		image::ImageLumaA8(_) => gl::RG,
		image::ImageRgb8(_) => gl::RGB,
		image::ImageRgba8(_) => gl::RGBA,
	};

	let data = img.raw_pixels();

	unsafe {
		gl::GenTextures(1, &mut texture_id);

		gl::BindTexture(gl::TEXTURE_2D, texture_id);
		gl::TexImage2D(gl::TEXTURE_2D, 0, format as i32, img.width() as i32, img.height() as i32, 0, format, gl::UNSIGNED_BYTE, &data[0] as *const u8 as *const std::os::raw::c_void);
		gl::GenerateMipmap(gl::TEXTURE_2D);

		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
	}

	texture_id
}