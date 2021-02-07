#![feature(destructuring_assignment)]

extern crate gl;
extern crate glfw;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_void;
use std::{mem, ptr};

extern crate image;
use crate::glfw::Context;
use gl::types::*;
use image::GenericImage;

extern crate cgmath;
use cgmath::{Matrix4, Vector3, vec3, point3, Deg, perspective};
use cgmath::prelude::*;

fn load_shader(source: std::string::String, _type: GLenum) -> u32 {
	let shader;
	let c_str = CString::new(source.as_bytes()).unwrap();
	unsafe {
		shader = gl::CreateShader(_type);
		gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
		gl::CompileShader(shader);
	}
	shader
}

macro_rules! c_str {
	($literal:expr) => {
		CStr::from_bytes_with_nul_unchecked(concat!($literal, "\0").as_bytes())
	};
}

fn compile_program(vertex_shader: u32, fragment_shader: u32) -> u32 {
	let program;
	unsafe {
		program = gl::CreateProgram();

		gl::AttachShader(program, vertex_shader);
		gl::AttachShader(program, fragment_shader);

		gl::LinkProgram(program);

		gl::DeleteShader(vertex_shader);
		gl::DeleteShader(fragment_shader);
	}
	program
}

fn prepare_gl() -> (u32, u32, u32, u32) {
	let vertices: [f32; 180] = [
		-0.5, -0.5, -0.5,  0.0, 0.0,
		 0.5, -0.5, -0.5,  1.0, 0.0,
		 0.5,  0.5, -0.5,  1.0, 1.0,
		 0.5,  0.5, -0.5,  1.0, 1.0,
		-0.5,  0.5, -0.5,  0.0, 1.0,
		-0.5, -0.5, -0.5,  0.0, 0.0,

		-0.5, -0.5,  0.5,  0.0, 0.0,
		 0.5, -0.5,  0.5,  1.0, 0.0,
		 0.5,  0.5,  0.5,  1.0, 1.0,
		 0.5,  0.5,  0.5,  1.0, 1.0,
		-0.5,  0.5,  0.5,  0.0, 1.0,
		-0.5, -0.5,  0.5,  0.0, 0.0,

		-0.5,  0.5,  0.5,  1.0, 0.0,
		-0.5,  0.5, -0.5,  1.0, 1.0,
		-0.5, -0.5, -0.5,  0.0, 1.0,
		-0.5, -0.5, -0.5,  0.0, 1.0,
		-0.5, -0.5,  0.5,  0.0, 0.0,
		-0.5,  0.5,  0.5,  1.0, 0.0,

		 0.5,  0.5,  0.5,  1.0, 0.0,
		 0.5,  0.5, -0.5,  1.0, 1.0,
		 0.5, -0.5, -0.5,  0.0, 1.0,
		 0.5, -0.5, -0.5,  0.0, 1.0,
		 0.5, -0.5,  0.5,  0.0, 0.0,
		 0.5,  0.5,  0.5,  1.0, 0.0,

		-0.5, -0.5, -0.5,  0.0, 1.0,
		 0.5, -0.5, -0.5,  1.0, 1.0,
		 0.5, -0.5,  0.5,  1.0, 0.0,
		 0.5, -0.5,  0.5,  1.0, 0.0,
		-0.5, -0.5,  0.5,  0.0, 0.0,
		-0.5, -0.5, -0.5,  0.0, 1.0,

		-0.5,  0.5, -0.5,  0.0, 1.0,
		 0.5,  0.5, -0.5,  1.0, 1.0,
		 0.5,  0.5,  0.5,  1.0, 0.0,
		 0.5,  0.5,  0.5,  1.0, 0.0,
		-0.5,  0.5,  0.5,  0.0, 0.0,
		-0.5,  0.5, -0.5,  0.0, 1.0
	];

	let vs_src = std::fs::read_to_string("triangle.vert")
		.expect("Error while reading vertex shader");
	let fs_src = std::fs::read_to_string("triangle.frag")
		.expect("Error while reading fragment shader");

	let vertex_shader = load_shader(vs_src, gl::VERTEX_SHADER);
	let fragment_shader = load_shader(fs_src, gl::FRAGMENT_SHADER);
	let shader_program = compile_program(vertex_shader, fragment_shader);

	let mut vao: u32 = 0;
	let mut vbo: u32 = 0;
	let mut texture: u32 = 0;
	let mut texture2: u32 = 0;

	unsafe {
		gl::Enable(gl::DEPTH_TEST);
		gl::Enable(gl::BLEND);
		gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);

		gl::GenBuffers(1, &mut vbo);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
			&vertices[0] as *const f32 as *const c_void,
			gl::STATIC_DRAW,
		);

		let stride = 5 * mem::size_of::<GLfloat>() as GLsizei;
		gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
		gl::EnableVertexAttribArray(0);

		gl::VertexAttribPointer(
			1,
			2,
			gl::FLOAT,
			gl::FALSE,
			stride,
			(3 * mem::size_of::<GLfloat>()) as *const c_void,
		);
		gl::EnableVertexAttribArray(1);

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

		let img = image::open(&std::path::Path::new("container.jpg"))
			.expect("Failed to load texture");
		let data = img.raw_pixels();
		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			gl::RGB as i32,
			img.width() as i32,
			img.height() as i32,
			0,
			gl::RGB,
			gl::UNSIGNED_BYTE,
			&data[0] as *const u8 as *const c_void,
		);
		gl::GenerateMipmap(gl::TEXTURE_2D);

		gl::GenTextures(1, &mut texture2);
		gl::BindTexture(gl::TEXTURE_2D, texture2);

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

		let img = image::open(&std::path::Path::new("awesomeface.png"))
			.expect("Failed to load second texture");
		let img = img.flipv();
		let data = img.raw_pixels();
		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			gl::RGB as i32,
			img.width() as i32,
			img.height() as i32,
			0,
			gl::RGBA,
			gl::UNSIGNED_BYTE,
			&data[0] as *const u8 as *const c_void,
		);
		gl::GenerateMipmap(gl::TEXTURE_2D);

		gl::UseProgram(shader_program);
		gl::Uniform1i(
			gl::GetUniformLocation(shader_program, c_str!("texture1").as_ptr()),
			0,
		);
		gl::Uniform1i(
			gl::GetUniformLocation(shader_program, c_str!("texture2").as_ptr()),
			1,
		);
	}

	(vao, shader_program, texture, texture2)
}

fn main() {
	let mut _glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

	_glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
	_glfw.window_hint(glfw::WindowHint::OpenGlProfile(
		glfw::OpenGlProfileHint::Core,
	));
	#[cfg(target_os = "macos")]
	_glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

	let (mut window, events) = _glfw
		.create_window(800, 600, "Learn OpenGL", glfw::WindowMode::Windowed)
		.expect("Failed to create window");

	window.make_current();
	window.set_key_polling(true);
	window.set_cursor_pos_polling(true);
	window.set_scroll_polling(true);
	window.set_framebuffer_size_polling(true);
	window.set_cursor_mode(glfw::CursorMode::Disabled);

	gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

	let vao: u32;
	let shader_program: u32;
	let texture: u32;
	let texture2: u32;
	(vao, shader_program, texture, texture2) = prepare_gl();

	let mut camera_pos = point3(0.0, 0.0, 3.0);
	let mut camera_front = vec3(0.0, 0.0, -1.0);
	let camera_up = vec3(0.0, 1.0, 0.0);
	let camera_speed = 2.5;

	let mut first_mouse = true;
	let mut yaw: f32 = -90.0;
	let mut pitch: f32 = 0.0;
	let mut last_x: f32 = 400.0;
	let mut last_y: f32 = 300.0;
	let mut fov: f32 = 45.0;

	let mut last_frame: f32 = 0.0;

	let cubes: [Vector3<f32>; 10] = [
		vec3( 0.0,  0.0,  0.0),
		vec3( 2.0,  5.0, -15.0),
		vec3(-1.5, -2.2, -2.5),
		vec3(-3.8, -2.0, -12.3),
		vec3( 2.4, -0.4, -3.5),
		vec3(-1.7,  3.0, -7.5),
		vec3( 1.3, -2.0, -2.5),
		vec3( 1.5,  2.0, -2.5),
		vec3( 1.5,  0.2, -1.5),
		vec3(-1.3,  1.0, -1.5)
	];

	while !window.should_close() {
		let current_frame = _glfw.get_time() as f32;
		let delta_time = current_frame - last_frame;
		last_frame = current_frame;

		for (_, event) in glfw::flush_messages(&events) {
			match event {
				glfw::WindowEvent::FramebufferSize(width, height) => unsafe {
					gl::Viewport(0, 0, width, height)
				},
				glfw::WindowEvent::Key(
					glfw::Key::Escape,
					_,
					glfw::Action::Release,
					_,
				) => window.set_should_close(true),
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

					let front = Vector3 {x: yaw.to_radians().cos() * pitch.to_radians().cos(),
					                     y: pitch.to_radians().sin(),
					                     z: yaw.to_radians().sin() * pitch.to_radians().cos()};
					camera_front = front.normalize();
				},
				glfw::WindowEvent::Scroll(_xoffset, yoffset) => {
					if fov >= 1.0 && fov <= 45.0 {
						fov -= yoffset as f32;
					}

					if fov <= 1.0 {
						fov = 1.0;
					}

					if fov >= 45.0 {
						fov = 45.0;
					}
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
			camera_pos -= camera_front.cross(camera_up).normalize() * camera_speed * delta_time;
		}
		if window.get_key(glfw::Key::D) == glfw::Action::Press {
			camera_pos += camera_front.cross(camera_up).normalize() * camera_speed * delta_time;
		}

		unsafe {
			let view = Matrix4::look_at_rh(camera_pos, camera_pos + camera_front, camera_up);

			let projection: Matrix4<f32> = perspective(Deg(fov), 800 as f32 / 600 as f32, 0.1, 100.0);

			let model_loc = gl::GetUniformLocation(shader_program, c_str!("model").as_ptr());
			let view_loc = gl::GetUniformLocation(shader_program, c_str!("view").as_ptr());
			let projection_loc = gl::GetUniformLocation(shader_program, c_str!("projection").as_ptr());

			gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
			gl::UniformMatrix4fv(projection_loc, 1, gl::FALSE, projection.as_ptr());
			gl::ClearColor(0.22, 0.18, 0.22, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, texture);
			gl::ActiveTexture(gl::TEXTURE1);
			gl::BindTexture(gl::TEXTURE_2D, texture2);
			gl::UseProgram(shader_program);
			gl::BindVertexArray(vao);
			for (i, cube) in cubes.iter().enumerate() {
				let mut model = Matrix4::from_translation(*cube);
				let angle = (_glfw.get_time() as f32) * 20.0 * (i + 1) as f32;
				model = model * Matrix4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), Deg(angle));
				gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
				gl::DrawArrays(gl::TRIANGLES, 0, 36);
			}
		}

		_glfw.poll_events();
		window.swap_buffers();
	}
}
