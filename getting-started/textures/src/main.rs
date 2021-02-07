#![feature(destructuring_assignment)]

extern crate gl;
extern crate glfw;
use std::ffi::CString;
use std::os::raw::c_void;
use std::{mem, ptr};

extern crate image;
use crate::glfw::Context;
use gl::types::*;
use image::GenericImage;

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

fn prepare_gl() -> (u32, u32, u32) {
	let vertices: [f32; 32] = [
		0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0,
		0.0, -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -0.5, 0.5, 0.0, 1.0, 1.0,
		0.0, 0.0, 1.0,
	];

	let indices: [u32; 6] = [0, 1, 3, 1, 2, 3];

	let vs_src = std::fs::read_to_string("triangle.vert")
		.expect("Error while reading vertex shader");
	let fs_src = std::fs::read_to_string("triangle.frag")
		.expect("Error while reading fragment shader");

	let vertex_shader = load_shader(vs_src, gl::VERTEX_SHADER);
	let fragment_shader = load_shader(fs_src, gl::FRAGMENT_SHADER);
	let shader_program = compile_program(vertex_shader, fragment_shader);

	let mut vao: u32 = 0;
	let mut vbo: u32 = 0;
	let mut ebo: u32 = 0;
	let mut texture: u32 = 0;

	unsafe {
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

		gl::GenBuffers(1, &mut ebo);
		gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
		gl::BufferData(
			gl::ELEMENT_ARRAY_BUFFER,
			(indices.len() * mem::size_of::<GLint>()) as GLsizeiptr,
			&indices[0] as *const u32 as *const c_void,
			gl::STATIC_DRAW,
		);

		let stride = 8 * mem::size_of::<GLfloat>() as GLsizei;
		gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
		gl::EnableVertexAttribArray(0);

		gl::VertexAttribPointer(
			1,
			3,
			gl::FLOAT,
			gl::FALSE,
			stride,
			(3 * mem::size_of::<GLfloat>()) as *const c_void,
		);
		gl::EnableVertexAttribArray(1);

		gl::VertexAttribPointer(
			2,
			2,
			gl::FLOAT,
			gl::FALSE,
			stride,
			(6 * mem::size_of::<GLfloat>()) as *const c_void,
		);
		gl::EnableVertexAttribArray(2);

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
	}

	(vao, shader_program, texture)
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
	window.set_framebuffer_size_polling(true);

	gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

	let vao: u32;
	let shader_program: u32;
	let texture: u32;
	(vao, shader_program, texture) = prepare_gl();

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
			gl::ClearColor(0.22, 0.18, 0.22, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::BindTexture(gl::TEXTURE_2D, texture);
			gl::UseProgram(shader_program);
			gl::BindVertexArray(vao);
			gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
		}

		_glfw.poll_events();
		window.swap_buffers();
	}
}
