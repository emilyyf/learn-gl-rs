extern crate gl;
extern crate glfw;
use std::ffi::CString;
use std::{mem, ptr};

use crate::glfw::Context;
use gl::types::*;

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
	window.set_framebuffer_size_polling(true);

	gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

	let vertices = vec![-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.5, 0.5, 0.0];

	let mut vbo = 0;
	let mut vao = 0;
	let mut vertex_shader = 0;
	let mut fragment_shader = 0;
	let mut shader_program = 0;
	let vs_src: &str = "#version 330 core
		layout (location = 0) in vec3 aPos;
		void main()
		{
				gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
		}";
	let fs_src: &str = "#version 330 core
		out vec4 FragColor;
		void main()
		{
				FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
		}";

	unsafe {
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);

		gl::GenBuffers(1, &mut vbo);
		gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
		gl::BufferData(
			gl::ARRAY_BUFFER,
			(vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
			mem::transmute(&vertices[0]),
			gl::STATIC_DRAW,
		);

		let c_str = CString::new(vs_src.as_bytes()).unwrap();
		vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
		gl::ShaderSource(vertex_shader, 1, &c_str.as_ptr(), ptr::null());
		gl::CompileShader(vertex_shader);

		let c_str2 = CString::new(fs_src.as_bytes()).unwrap();
		fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
		gl::ShaderSource(fragment_shader, 1, &c_str2.as_ptr(), ptr::null());
		gl::CompileShader(fragment_shader);

		shader_program = gl::CreateProgram();

		gl::AttachShader(shader_program, vertex_shader);
		gl::AttachShader(shader_program, fragment_shader);

		gl::LinkProgram(shader_program);
		gl::UseProgram(shader_program);

		gl::VertexAttribPointer(
			0,
			3,
			gl::FLOAT,
			gl::FALSE,
			3 * mem::size_of::<GLfloat>() as GLsizei,
			ptr::null(),
		);
		gl::EnableVertexAttribArray(0);

		gl::BindBuffer(gl::ARRAY_BUFFER, 0);
		gl::BindVertexArray(0);
	}

	while !window.should_close() {
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
				_ => {}
			}
		}

		unsafe {
			gl::ClearColor(1.0, 0.0, 0.0, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::UseProgram(shader_program);
			gl::BindVertexArray(vao);
			gl::DrawArrays(gl::TRIANGLES, 0, 3);
		}

		_glfw.poll_events();
		window.swap_buffers();
	}

	unsafe {
		gl::DeleteShader(vertex_shader);
		gl::DeleteShader(fragment_shader);
	}
}
