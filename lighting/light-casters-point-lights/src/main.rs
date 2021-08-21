#![feature(destructuring_assignment)]
#![feature(or_patterns)]

extern crate cgmath;
extern crate gl;
extern crate glfw;
extern crate image;

mod shader;
use crate::glfw::Context;
use cgmath::prelude::*;
use image::GenericImage;

fn gl_init() -> (
	glfw::Glfw,
	glfw::Window,
	std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
) {
	let mut glfw: glfw::Glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

	glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
	glfw.window_hint(glfw::WindowHint::OpenGlProfile(
		glfw::OpenGlProfileHint::Core,
	));
	#[cfg(target_os = "macos")]
	_glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

	let (mut window, events) = glfw
		.create_window(800, 600, "Learn OpenGL", glfw::WindowMode::Windowed)
		.expect("Failed to create window");

	window.make_current();
	window.set_key_polling(true);
	window.set_cursor_pos_polling(true);
	window.set_scroll_polling(true);
	window.set_framebuffer_size_polling(true);
	window.set_cursor_mode(glfw::CursorMode::Disabled);

	gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

	unsafe {
		gl::Enable(gl::DEPTH_TEST);
		gl::Enable(gl::BLEND);
		gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
	}

	(glfw, window, events)
}

fn create_vao() -> (u32, u32) {
	let mut object_vao: u32 = 0;
	let mut light_vao: u32 = 0;
	let mut vbo: u32 = 0;

	unsafe {
		gl::GenVertexArrays(1, &mut object_vao);
		gl::BindVertexArray(object_vao);

		gl::GenBuffers(1, &mut vbo);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(CUBE_VERTICES.len() * std::mem::size_of::<gl::types::GLfloat>())
				as gl::types::GLsizeiptr,
			&CUBE_VERTICES[0] as *const f32 as *const std::os::raw::c_void,
			gl::STATIC_DRAW,
		);

		let stride =
			8 * std::mem::size_of::<gl::types::GLfloat>() as gl::types::GLsizei;
		gl::VertexAttribPointer(
			0,
			3,
			gl::FLOAT,
			gl::FALSE,
			stride,
			std::ptr::null(),
		);
		gl::EnableVertexAttribArray(0);

		gl::VertexAttribPointer(
			1,
			3,
			gl::FLOAT,
			gl::FALSE,
			stride,
			(3 * std::mem::size_of::<gl::types::GLfloat>())
				as *const std::os::raw::c_void,
		);
		gl::EnableVertexAttribArray(1);

		gl::VertexAttribPointer(
			2,
			2,
			gl::FLOAT,
			gl::FALSE,
			stride,
			(6 * std::mem::size_of::<gl::types::GLfloat>())
				as *const std::os::raw::c_void,
		);
		gl::EnableVertexAttribArray(2);

		gl::GenVertexArrays(1, &mut light_vao);
		gl::BindVertexArray(light_vao);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::VertexAttribPointer(
			0,
			3,
			gl::FLOAT,
			gl::FALSE,
			stride,
			std::ptr::null(),
		);
		gl::EnableVertexAttribArray(0);
	}

	(object_vao, light_vao)
}

fn load_texture(path: &str) -> u32 {
	let mut texture: u32 = 0;
	let img =
		image::open(&std::path::Path::new(path)).expect("Failed to load texture");
	let data = img.raw_pixels();
	let format = match img {
		image::ImageLuma8(_) => gl::RED,
		image::ImageLumaA8(_) => gl::RG,
		image::ImageRgb8(_) => gl::RGB,
		image::ImageRgba8(_) => gl::RGBA,
	};

	unsafe {
		gl::GenTextures(1, &mut texture);
		gl::BindTexture(gl::TEXTURE_2D, texture);

		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
		gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
		gl::TexParameteri(
			gl::TEXTURE_2D,
			gl::TEXTURE_MIN_FILTER,
			gl::LINEAR as i32,
		);
		gl::TexParameteri(
			gl::TEXTURE_2D,
			gl::TEXTURE_MAG_FILTER,
			gl::LINEAR as i32,
		);

		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			format as i32,
			img.width() as i32,
			img.height() as i32,
			0,
			format,
			gl::UNSIGNED_BYTE,
			&data[0] as *const u8 as *const std::os::raw::c_void,
		);

		gl::GenerateMipmap(gl::TEXTURE_2D);
	}

	texture
}

fn main() {
	let (mut glfw, mut window, events) = gl_init();
	let mut camera_pos = cgmath::point3(0.0, 0.0, 6.0);
	let mut camera_front = cgmath::vec3(0.0, 0.0, -1.0);
	let camera_up = cgmath::vec3(0.0, 1.0, 0.0);
	let camera_speed = 2.5;
	let mut last_frame: f32 = 0.0;
	let mut first_mouse = true;
	let mut yaw: f32 = -90.0;
	let mut pitch: f32 = 0.0;
	let mut last_x: f32 = 400.0;
	let mut last_y: f32 = 300.0;

	let light_pos = cgmath::vec3(1.2, 1.0, 2.0);
	let light_shader = shader::create_shader("light.vert", "light.frag");

	let object_shader = shader::create_shader("object.vert", "object.frag");

	let (object_vao, light_vao) = create_vao();
	let diffuse_map = load_texture("container2.png");
	let specular_map = load_texture("container2_specular.png");

	let cubes: [cgmath::Vector3<f32>; 10] = [
		cgmath::vec3(0.0, 0.0, 0.0),
		cgmath::vec3(2.0, 5.0, -15.0),
		cgmath::vec3(-1.5, -2.2, -2.5),
		cgmath::vec3(-3.8, -2.0, -12.3),
		cgmath::vec3(2.4, -0.4, -3.5),
		cgmath::vec3(-1.7, 3.0, -7.5),
		cgmath::vec3(1.3, -2.0, -2.5),
		cgmath::vec3(1.5, 2.0, -2.5),
		cgmath::vec3(1.5, 0.2, -1.5),
		cgmath::vec3(-1.3, 1.0, -1.5),
	];

	while !window.should_close() {
		let current_frame = glfw.get_time() as f32;
		let delta_time = current_frame - last_frame;
		last_frame = current_frame;

		for (_, event) in glfw::flush_messages(&events) {
			match event {
				glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
					gl::Viewport(0, 0, width, height)
				},
				glfw::WindowEvent::Key(
					glfw::Key::Escape | glfw::Key::Q,
					_,
					glfw::Action::Release,
					_,
				) => {
					window.set_should_close(true);
				}
				glfw::WindowEvent::CursorPos(xpos, ypos) => {
					let (xpos, ypos) = (xpos as f32, ypos as f32);
					if first_mouse {
						last_x = xpos;
						last_y = ypos;
						first_mouse = false;
					}

					let mut xoffset = xpos - last_x;
					let mut yoffset = last_y - ypos;
					last_x = xpos;
					last_y = ypos;

					let sensitivity: f32 = 0.1;
					xoffset *= sensitivity;
					yoffset *= sensitivity;

					yaw += xoffset;
					pitch += yoffset;

					if pitch > 89.0 {
						pitch = 89.0;
					}
					if pitch < -89.0 {
						pitch = -89.0;
					}

					let front = cgmath::Vector3 {
						x: yaw.to_radians().cos() * pitch.to_radians().cos(),
						y: pitch.to_radians().sin(),
						z: yaw.to_radians().sin() * pitch.to_radians().cos(),
					};
					camera_front = front.normalize();
				}
				_ => {}
			}
		}

		if window.get_key(glfw::Key::W) == glfw::Action::Press {
			camera_pos += camera_speed * camera_front * delta_time;
		}
		if window.get_key(glfw::Key::S) == glfw::Action::Press {
			camera_pos -= camera_speed * camera_front * delta_time;
		}
		if window.get_key(glfw::Key::A) == glfw::Action::Press {
			camera_pos -=
				camera_front.cross(camera_up).normalize() * camera_speed * delta_time;
		}
		if window.get_key(glfw::Key::D) == glfw::Action::Press {
			camera_pos +=
				camera_front.cross(camera_up).normalize() * camera_speed * delta_time;
		}

		unsafe {
			let view = cgmath::Matrix4::look_at_rh(
				camera_pos,
				camera_pos + camera_front,
				camera_up,
			);
			let projection: cgmath::Matrix4<f32> = cgmath::perspective(
				cgmath::Deg(45.0),
				800 as f32 / 600 as f32,
				0.1,
				100.0,
			);

			gl::ClearColor(0.22, 0.18, 0.22, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

			gl::UseProgram(object_shader);

			let light_color: cgmath::Vector3<f32> = cgmath::vec3(1.0, 1.0, 1.0);
			let diffuse_color = light_color * 0.5;
			let ambient_color = light_color * 0.2;

			shader::set_vec3(
				object_shader,
				"material.ambient",
				cgmath::vec3(1.0, 0.5, 0.31),
			);
			shader::set_int(object_shader, "material.diffuse", 0);
			shader::set_int(object_shader, "material.specular", 1);
			shader::set_float(object_shader, "material.shininess", 32.0);
			shader::set_vec3(object_shader, "light.position", light_pos);
			shader::set_vec3(object_shader, "light.ambient", ambient_color);
			shader::set_vec3(object_shader, "light.diffuse", diffuse_color);
			shader::set_vec3(
				object_shader,
				"light.specular",
				cgmath::vec3(1.0, 1.0, 1.0),
			);
			shader::set_float(object_shader, "light.constant", 1.0);
			shader::set_float(object_shader, "light.linear", 0.09);
			shader::set_float(object_shader, "light.quadratic", 0.032);
			shader::set_point3(object_shader, "viewPos", camera_pos);
			shader::set_mat4(object_shader, "view", view);
			shader::set_mat4(object_shader, "projection", projection);
			gl::BindVertexArray(object_vao);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, diffuse_map);
			gl::ActiveTexture(gl::TEXTURE1);
			gl::BindTexture(gl::TEXTURE_2D, specular_map);
			for (i, cube) in cubes.iter().enumerate() {
				let mut model = cgmath::Matrix4::from_translation(*cube);
				let angle = 20.0 * i as f32;
				model = model
					* cgmath::Matrix4::from_axis_angle(
						cgmath::vec3(1.0, 0.3, 0.5).normalize(),
						cgmath::Deg(angle),
					);
				shader::set_mat4(object_shader, "model", model);
				gl::DrawArrays(gl::TRIANGLES, 0, 36);
			}

			gl::UseProgram(light_shader);
			shader::set_mat4(light_shader, "view", view);
			shader::set_mat4(light_shader, "projection", projection);
			let mut model = cgmath::Matrix4::from_translation(light_pos);
			model = model * cgmath::Matrix4::from_scale(0.2);
			shader::set_mat4(light_shader, "model", model);

			gl::BindVertexArray(light_vao);
			gl::DrawArrays(gl::TRIANGLES, 0, 36);
		}

		glfw.poll_events();
		window.swap_buffers();
	}
}

static CUBE_VERTICES: [f32; 288] = [
	-0.5, -0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 0.0, 0.5, -0.5, -0.5, 0.0, 0.0, -1.0,
	1.0, 0.0, 0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 1.0, 1.0, 0.5, 0.5, -0.5, 0.0, 0.0,
	-1.0, 1.0, 1.0, -0.5, 0.5, -0.5, 0.0, 0.0, -1.0, 0.0, 1.0, -0.5, -0.5, -0.5,
	0.0, 0.0, -1.0, 0.0, 0.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5,
	-0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0,
	0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 1.0, 1.0, -0.5, 0.5, 0.5, 0.0, 0.0, 1.0, 0.0,
	1.0, -0.5, -0.5, 0.5, 0.0, 0.0, 1.0, 0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0,
	0.0, 1.0, 0.0, -0.5, 0.5, -0.5, -1.0, 0.0, 0.0, 1.0, 1.0, -0.5, -0.5, -0.5,
	-1.0, 0.0, 0.0, 0.0, 1.0, -0.5, -0.5, -0.5, -1.0, 0.0, 0.0, 0.0, 1.0, -0.5,
	-0.5, 0.5, -1.0, 0.0, 0.0, 0.0, 0.0, -0.5, 0.5, 0.5, -1.0, 0.0, 0.0, 1.0,
	0.0, 0.5, 0.5, 0.5, 1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 0.5, -0.5, 1.0, 0.0, 0.0,
	1.0, 1.0, 0.5, -0.5, -0.5, 1.0, 0.0, 0.0, 0.0, 1.0, 0.5, -0.5, -0.5, 1.0,
	0.0, 0.0, 0.0, 1.0, 0.5, -0.5, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5,
	1.0, 0.0, 0.0, 1.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0, 0.5,
	-0.5, -0.5, 0.0, -1.0, 0.0, 1.0, 1.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0,
	0.0, 0.5, -0.5, 0.5, 0.0, -1.0, 0.0, 1.0, 0.0, -0.5, -0.5, 0.5, 0.0, -1.0,
	0.0, 0.0, 0.0, -0.5, -0.5, -0.5, 0.0, -1.0, 0.0, 0.0, 1.0, -0.5, 0.5, -0.5,
	0.0, 1.0, 0.0, 0.0, 1.0, 0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 1.0, 1.0, 0.5, 0.5,
	0.5, 0.0, 1.0, 0.0, 1.0, 0.0, 0.5, 0.5, 0.5, 0.0, 1.0, 0.0, 1.0, 0.0, -0.5,
	0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0, -0.5, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 1.0,
];
