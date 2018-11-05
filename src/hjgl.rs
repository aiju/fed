use gl::types::*;
use std::ffi::CString;

pub fn refcall<T, U: Fn(*mut T) -> ()>(f: U) -> T {
	let mut x = unsafe { std::mem::uninitialized() };
	f(&mut x);
	x
}
pub fn cstr(s: &str) -> CString {
	CString::new(s).expect("null byte in string")
}

pub struct Shader {
	id: GLuint,
}
impl Shader {
	pub fn new(s: &str, typ: u32) -> Result<Shader,String> {
		unsafe {
			let id = gl::CreateShader(typ);
			let sp = s.as_bytes().as_ptr() as *const i8;
			let slen = s.len() as i32;
			gl::ShaderSource(id, 1, &sp, &slen);
			gl::CompileShader(id);
			let status = refcall(|x| gl::GetShaderiv(id, gl::COMPILE_STATUS, x));
			if status == gl::FALSE as GLint {
				let len = refcall(|x| gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, x));
				let mut s = vec![0 as u8; len as usize];
				gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), s.as_mut_ptr() as *mut i8);
				gl::DeleteShader(id);
				return Err(String::from_utf8(s).unwrap())
			}
			Ok(Shader { id })
		}
	}
}
impl Drop for Shader {
	fn drop(&mut self) {
		unsafe { gl::DeleteShader(self.id); }
	}
}

pub struct Program {
	id: GLuint
}
impl Program {
	pub fn new(parts: &[Shader]) -> Result<Program,String> {
		unsafe {
			let id = gl::CreateProgram();
			for p in parts {
				gl::AttachShader(id, p.id);
			}
			gl::LinkProgram(id);
			let status = refcall(|x| gl::GetProgramiv(id, gl::LINK_STATUS, x));
			if status == gl::FALSE as GLint {
				let len = refcall(|x| gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, x));
				let mut s = vec![0 as u8; len as usize];
				gl::GetProgramInfoLog(id, len, std::ptr::null_mut(), s.as_mut_ptr() as *mut i8);
				gl::DeleteProgram(id);
				return Err(String::from_utf8(s).unwrap())
			}
			Ok(Program { id })
		}
	}
	pub fn get_uniform_location(&self, name: &str) -> Option<GLuint> {
		let cname = cstr(name);
		let g = unsafe { gl::GetUniformLocation(self.id, cname.as_ptr()) };
		if g < 0 {
			None
		} else {
			Some(g as GLuint)
		}
	}
	pub fn get_attrib_location(&self, name: &str) -> Option<GLuint> {
		let cname = cstr(name);
		let g = unsafe { gl::GetAttribLocation(self.id, cname.as_ptr()) };
		if g < 0 {
			None
		} else {
			Some(g as GLuint)
		}
	}
	pub fn bind(&self) {
		unsafe { gl::UseProgram(self.id) }
	}
}
impl Drop for Program {
	fn drop(&mut self) {
		unsafe { gl::DeleteProgram(self.id); }
	}
}

pub struct Texture {
	id: GLuint,
}
impl Texture {
	pub fn new() -> Texture {
		Texture { id : refcall(|x| unsafe { gl::GenTextures(1, x) }) }
	}
	pub fn bind(&self) {
		unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id); }
	}
	pub fn id(&self) -> GLuint {
		self.id
	}
}
impl Drop for Texture {
	fn drop(&mut self) {
		unsafe { gl::DeleteTextures(1, &self.id); }
	}
}

pub struct VBO {
	id: GLuint,
}
impl VBO {
	pub fn new() -> VBO {
		let mut rc : GLuint = 0;
		unsafe { gl::GenBuffers(1, &mut rc); }
		VBO { id : rc }
	}
	pub fn bind(&self, targ: u32) {
		unsafe { gl::BindBuffer(targ, self.id); }
	}
}
impl Drop for VBO {
	fn drop(&mut self) {
		unsafe { gl::DeleteBuffers(1, &self.id); }
	}
}

pub struct VAO {
	id: GLuint,
}
impl VAO {
	pub fn new() -> VAO {
		let mut rc : GLuint = 0;
		unsafe { gl::GenVertexArrays(1, &mut rc); }
		VAO { id : rc }
	}
	pub fn bind(&self) {
		unsafe { gl::BindVertexArray(self.id); }
	}
}
impl Drop for VAO {
	fn drop(&mut self) {
		unsafe { gl::DeleteVertexArrays(1, &self.id); }
	}
}

