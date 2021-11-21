#![feature(destructuring_assignment)]

extern crate cgmath;
extern crate gl;
extern crate glfw;
extern crate image;

mod shader;
mod model;
use crate::glfw::Context;
use cgmath::prelude::*;

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
	glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

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

	let object_shader = shader::create_shader("object.vert", "object.frag");

	let model = model::load_model("../../assets/terrain/terrain.obj");

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

			shader::set_mat4(object_shader, "projection", projection);
			shader::set_mat4(object_shader, "view", view);
			let mut model_: cgmath::Matrix4<f32> = cgmath::Matrix4::from_translation(cgmath::vec3(0.0, -1.75, 0.0));
			model_ = model_ * cgmath::Matrix4::from_scale(10.0);
			shader::set_mat4(object_shader, "model", model_);
			model::draw_model(&model, object_shader);
		}

		glfw.poll_events();
		window.swap_buffers();
	}
}
