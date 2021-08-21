use cgmath::prelude::*;
use gl::types::*;

fn load_shader(source: std::string::String, _type: GLenum) -> u32 {
	let shader;
	let c_str = std::ffi::CString::new(source.as_bytes()).unwrap();

	unsafe {
		shader = gl::CreateShader(_type);
		gl::ShaderSource(shader, 1, &c_str.as_ptr(), std::ptr::null());
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

pub fn create_shader(vert_path: &str, frag_path: &str) -> u32 {
	let vs_src = std::fs::read_to_string(vert_path)
		.expect("Error while reading vertex shader");
	let fs_src = std::fs::read_to_string(frag_path)
		.expect("Error while reading fragment shader");

	let vertex_shader = load_shader(vs_src, gl::VERTEX_SHADER);
	let fragment_shader = load_shader(fs_src, gl::FRAGMENT_SHADER);
	let shader_program = compile_program(vertex_shader, fragment_shader);

	shader_program
}

pub fn set_mat4(program: u32, name: &str, data: cgmath::Matrix4<f32>) {
	let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();

	unsafe {
		let location = gl::GetUniformLocation(program, c_str.as_ptr());

		gl::UniformMatrix4fv(location, 1, gl::FALSE, data.as_ptr());
	}
}

pub fn set_vec3(program: u32, name: &str, data: cgmath::Vector3<f32>) {
	let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();

	unsafe {
		let location = gl::GetUniformLocation(program, c_str.as_ptr());

		gl::Uniform3fv(location, 1, data.as_ptr());
	}
}
