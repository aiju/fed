use dat::*;
use imgui_sys::*;
use hjgl::*;
use sdl2::keyboard::Scancode;
use std::os::raw::*;
use gl::types::*;
use std::time::Instant;

macro_rules! offset_of {
	($ty:ty, $field:ident) => {
		&(*(0 as *const $ty)).$field as *const _ as usize
	}
}

const VERTEX_SHADER: &str = r#"
	uniform mat4 ProjMtx;
	attribute vec2 Position;
	attribute vec2 UV;
	attribute vec4 Color;
	varying vec2 Frag_UV;
	varying vec4 Frag_Color;
	void main()
	{
	    Frag_UV = UV;
	    Frag_Color = Color;
	    gl_Position = ProjMtx * vec4(Position.xy,0,1);
	}
"#;
const FRAGMENT_SHADER: &str = r#"
	uniform sampler2D Texture;
	varying vec2 Frag_UV;
	varying vec4 Frag_Color;
	void main()
	{
	    gl_FragColor = Frag_Color * texture2D(Texture, Frag_UV.st);
	};
"#;

pub enum DrawCmd {
	Circle(Vec2, f32, Color, f32),
	CircleFilled(Vec2, f32, Color),
	Line(Vec2, Vec2, Color, f32),
	Rect(Vec2, Vec2, Color, f32),
	RectFilled(Vec2, Vec2, Color),
}

pub struct Imgui {
	_fonts: Texture,
	lastframe: Instant,
	w: f32,
	h: f32,
	vao: VAO,
	vert: VBO,
	elem: VBO,
	prog: Program,
	locprojmtx: GLuint,
}
impl Imgui {
	pub fn new(w: f32, h: f32) -> Imgui {
		unsafe {
			igCreateContext(None, None);
			let io = igGetIO();
			
			(*io).key_map[ImGuiKey::Tab as usize] = Scancode::Tab as i32;
			(*io).key_map[ImGuiKey::LeftArrow as usize] = Scancode::Left as i32;
			(*io).key_map[ImGuiKey::RightArrow as usize] = Scancode::Right as i32;
			(*io).key_map[ImGuiKey::UpArrow as usize] = Scancode::Up as i32;
			(*io).key_map[ImGuiKey::DownArrow as usize] = Scancode::Down as i32;
			(*io).key_map[ImGuiKey::PageUp as usize] = Scancode::PageUp as i32;
			(*io).key_map[ImGuiKey::PageDown as usize] = Scancode::PageDown as i32;
			(*io).key_map[ImGuiKey::Home as usize] = Scancode::Home as i32;
			(*io).key_map[ImGuiKey::End as usize] = Scancode::End as i32;
			(*io).key_map[ImGuiKey::Delete as usize] = Scancode::Delete as i32;
			(*io).key_map[ImGuiKey::Backspace as usize] = Scancode::Backspace as i32;
			(*io).key_map[ImGuiKey::Enter as usize] = Scancode::Return as i32;
			(*io).key_map[ImGuiKey::Escape as usize] = Scancode::Escape as i32;
			(*io).key_map[ImGuiKey::A as usize] = Scancode::A as i32;
			(*io).key_map[ImGuiKey::C as usize] = Scancode::C as i32;
			(*io).key_map[ImGuiKey::V as usize] = Scancode::V as i32;
			(*io).key_map[ImGuiKey::X as usize] = Scancode::X as i32;
			(*io).key_map[ImGuiKey::Y as usize] = Scancode::Y as i32;
			(*io).key_map[ImGuiKey::Z as usize] = Scancode::Z as i32;
			
			gl::Enable(gl::BLEND);
			gl::BlendEquation(gl::FUNC_ADD);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
			gl::Disable(gl::CULL_FACE);
			gl::Disable(gl::DEPTH_TEST);
			gl::Enable(gl::SCISSOR_TEST);

			let mut pixels : *mut c_uchar = std::ptr::null_mut();
			let mut width : c_int = 0;
			let mut height : c_int = 0;
			let mut bpp : c_int = 0;
			ImFontAtlas_GetTexDataAsRGBA32((*io).fonts, &mut pixels, &mut width, &mut height, &mut bpp);

			let fonts = Texture::new();

			fonts.bind();
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
			gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
			gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width, height, 0, gl::RGBA, gl::UNSIGNED_BYTE, pixels as *const c_void);
			(*(*io).fonts).tex_id = fonts.id() as *mut c_void;
			
			let vshad = Shader::new(VERTEX_SHADER, gl::VERTEX_SHADER).unwrap();
			let fshad = Shader::new(FRAGMENT_SHADER, gl::FRAGMENT_SHADER).unwrap();
			let prog = Program::new(&vec![vshad, fshad]).unwrap();
			
			let loctex = prog.get_uniform_location("Texture").unwrap();
			let locprojmtx = prog.get_uniform_location("ProjMtx").unwrap();
			let locpos = prog.get_attrib_location("Position").unwrap();
			let locuv = prog.get_attrib_location("UV").unwrap();
			let loccol = prog.get_attrib_location("Color").unwrap();
			
			prog.bind();
			gl::Uniform1i(loctex as i32, 0);
			
			let vao = VAO::new();
			let vert = VBO::new();
			
			vao.bind();
			vert.bind(gl::ARRAY_BUFFER);
			gl::EnableVertexAttribArray(locpos);
			gl::EnableVertexAttribArray(locuv);
			gl::EnableVertexAttribArray(loccol);
			gl::VertexAttribPointer(locpos, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<ImDrawVert>() as i32, offset_of!(ImDrawVert, pos) as *const c_void);
			gl::VertexAttribPointer(locuv, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<ImDrawVert>() as i32, offset_of!(ImDrawVert, uv) as *const c_void);
			gl::VertexAttribPointer(loccol, 4, gl::UNSIGNED_BYTE, gl::TRUE, std::mem::size_of::<ImDrawVert>() as i32, offset_of!(ImDrawVert, col) as *const c_void);

			Imgui {
				_fonts: fonts,
				lastframe: Instant::now(),
				w, h,
				vao,
				vert,
				elem: VBO::new(),
				prog,
				locprojmtx,
			}
		}
	}
	pub fn frame(&mut self) {
		unsafe {
			let io = igGetIO();
			let now = Instant::now();
			let delta = now - self.lastframe;
			(*io).delta_time = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
			self.lastframe = now;

			(*io).display_size.x = self.w;
			(*io).display_size.y = self.h;
			
			igNewFrame();
		}
	}
	pub fn render(&self) {
		unsafe {
			igEndFrame();
			igRender();
			
			let io = igGetIO();
			
			gl::Viewport(0, 0, self.w as i32, self.h as i32);
			gl::Scissor(0, 0, self.w as i32, self.h as i32);
			gl::ClearColor(0.5, 0.3, 0.4, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			
			self.vao.bind();
			self.prog.bind();
			let l = 0.0;
			let r = (*io).display_size.x;
			let t = 0.0;
			let b = (*io).display_size.y;
			let matrix = [
				2.0/(r-l), 0.0, 0.0, 0.0,
				0.0, 2.0/(t-b), 0.0, 0.0,
				0.0, 0.0, -1.0, 0.0,
				(r+l)/(l-r), (t+b)/(b-t), 0.0, 1.0
			];
			gl::UniformMatrix4fv(self.locprojmtx as i32, 1, gl::FALSE, matrix.as_ptr());
			
			let draw_data = igGetDrawData();
			let cmd_lists : &[*mut ImDrawList] = std::slice::from_raw_parts((*draw_data).cmd_lists, (*draw_data).cmd_lists_count as usize);
			for &l in cmd_lists {
				let vtx = &(*l).vtx_buffer;
				let idx = &(*l).idx_buffer;
				
				self.vert.bind(gl::ARRAY_BUFFER);
				gl::BufferData(gl::ARRAY_BUFFER,
					vtx.size as isize * std::mem::size_of::<ImDrawVert>() as isize,
					vtx.data as *const c_void,
					gl::STREAM_DRAW);
				self.elem.bind(gl::ELEMENT_ARRAY_BUFFER);
				gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
					idx.size as isize * std::mem::size_of::<ImDrawIdx>() as isize,
					idx.data as *const c_void,
					gl::STREAM_DRAW);
				let mut idxoffset = 0;

				for c in (*l).cmd_buffer.as_slice() {
					if let Some(cb) = c.user_callback {
						cb(l, c);
					} else {
						let r = c.clip_rect;
						if r.x < self.w && r.y < self.h && r.z >= 0.0 && r.w >= 0.0 {
							gl::BindTexture(gl::TEXTURE_2D, c.texture_id as u32);
							gl::Scissor(
								c.clip_rect.x as i32,
								(self.h - c.clip_rect.w) as i32,
								(c.clip_rect.z - c.clip_rect.x) as i32,
								(c.clip_rect.w - c.clip_rect.y) as i32);
							gl::DrawElements(gl::TRIANGLES,
								c.elem_count as i32, 
								gl::UNSIGNED_SHORT,
								idxoffset as *const c_void);
						}
					}
					idxoffset += c.elem_count * 2;
				}
			}
		}
	}
	pub fn mouse(&self, x: i32, y: i32, l: bool, r: bool) {
		unsafe {
			let io = igGetIO();
			(*io).mouse_pos.x = x as f32;
			(*io).mouse_pos.y = y as f32;
			(*io).mouse_down[0] = l;
			(*io).mouse_down[1] = r;
		}
	}
	pub fn keyboard(&self, kb: sdl2::keyboard::KeyboardState, keymod: sdl2::keyboard::Mod) {
		unsafe {
			let io = igGetIO();
			for (i,s) in kb.scancodes() {
				if (i as usize) < 512 {
					(*io).keys_down[i as usize] = s;
				}
			}
			(*io).key_ctrl = keymod.contains(sdl2::keyboard::LCTRLMOD) || keymod.contains(sdl2::keyboard::RCTRLMOD);
			(*io).key_shift = keymod.contains(sdl2::keyboard::LSHIFTMOD) || keymod.contains(sdl2::keyboard::RSHIFTMOD);
		}
	}
	pub fn add_text(&self, text: &str) {
		unsafe {
			let ctext = cstr(text);
			let io = igGetIO();
			ImGuiIO_AddInputCharactersUTF8(ctext.as_ptr());
		}
	}
		
	pub fn text(&self, s: &str) {
		let fmt = cstr("%s");
		let cs = cstr(s);
		unsafe { igText(fmt.as_ptr(), cs.as_ptr()); }
	}
	
	pub fn end(&self) {
		unsafe { igEnd(); }
	}
	
	pub fn invisible_button(&self, s: &str, size: Vec2) -> bool {
		let cs = cstr(s);
		unsafe { igInvisibleButton(cs.as_ptr(), ImVec2::new(size.x, size.y)) }
	}

	pub fn button(&self, s: &str, size: Vec2) -> bool {
		let cs = cstr(s);
		unsafe { igButton(cs.as_ptr(), ImVec2::new(size.x, size.y)) }
	}

	pub fn radio_button(&self, s: &str, state: bool) -> bool {
		let cs = cstr(s);
		unsafe { igRadioButtonBool(cs.as_ptr(), state) }
	}

	pub fn input_text(&self, s: &str, b: &mut ImguiBuf) -> bool {
		unsafe {
			let cs = cstr(s);
			igInputText(cs.as_ptr(), b.as_ptr(), b.len(), ImGuiInputTextFlags::empty(), None, std::ptr::null_mut())
		}
	}
	
	pub fn draw(&self, l: &[DrawCmd], p: Vec2) {
		let drawlist = unsafe { igGetWindowDrawList() };
		for i in l {
			match i {
			DrawCmd::Circle(c, r, col, thick) =>
				unsafe { ImDrawList_AddCircle(drawlist, (*c+p).imvec(), *r, col.to_u32(), 32, *thick) },
			DrawCmd::CircleFilled(c, r, col) =>
				unsafe { ImDrawList_AddCircleFilled(drawlist, (*c+p).imvec(), *r, col.to_u32(), 32) },
			DrawCmd::Line(a, b, col, thick) =>
				unsafe { ImDrawList_AddLine(drawlist, (*a+p).imvec(), (*b+p).imvec(), col.to_u32(), *thick) },
			DrawCmd::Rect(a, b, col, thick) =>
				unsafe { ImDrawList_AddRect(drawlist, (*a+p).imvec(), (*b+p).imvec(), col.to_u32(), 0.0, ImDrawCornerFlags::empty(), *thick) },
			DrawCmd::RectFilled(a, b, col) =>
				unsafe { ImDrawList_AddRectFilled(drawlist, (*a+p).imvec(), (*b+p).imvec(), col.to_u32(), 0.0, ImDrawCornerFlags::empty()) },
			}
		}
	}
	
	pub fn is_item_hovered(&self) -> bool {
		unsafe { igIsItemHovered(ImGuiHoveredFlags::empty()) }
	}

	pub fn is_mouse_clicked(&self, x: i32) -> bool {
		unsafe { igIsMouseClicked(x, false) }
	}
	
	pub fn is_mouse_released(&self, x: i32) -> bool {
		unsafe { igIsMouseReleased(x) }
	}
	
	pub fn is_mouse_down(&self, x: i32) -> bool {
		unsafe { igIsMouseDown(x) }
	}
	
	pub fn is_key_down(&self, kc: Scancode) -> bool {
		unsafe {
			let io = igGetIO();
			(kc as usize) < 512 && (*io).keys_down[kc as usize]
		}
	}
	
	pub fn is_ctrl_down(&self) -> bool {
		unsafe {
			let io = igGetIO();
			(*io).key_ctrl
		}
	}
	
	pub fn mouse_pos(&self) -> Vec2 {
		unsafe {
			let io = igGetIO();
			let p = (*io).mouse_pos;
			Vec2::new(p.x, p.y)
		}
	}
	
	pub fn cursor_screen_pos(&self) -> Vec2 {
		let p = unsafe { refcall(|x| igGetCursorScreenPos(x)) };
		Vec2::new(p.x, p.y)
	}
}

pub struct ImguiBegin<'a> {
	title: &'a str,
	flags: ImGuiWindowFlags,
}

impl Imgui {
	pub fn window<'a>(&self, s: &'a str) -> ImguiBegin<'a> {
		ImguiBegin {
			title: s,
			flags: ImGuiWindowFlags::empty(),
		}
	}
}
impl<'a> ImguiBegin<'a> {
	pub fn size(&'a mut self, w: f32, h: f32) -> &'a mut ImguiBegin {
		unsafe { igSetNextWindowSize(ImVec2::new(w, h), ImGuiCond::Once); }
		self
	}
	pub fn pos(&'a mut self, w: f32, h: f32) -> &'a mut ImguiBegin {
		unsafe { igSetNextWindowPos(ImVec2::new(w, h), ImGuiCond::Once, ImVec2::zero()); }
		self
	}
	pub fn begin(&self) {
		let cs = cstr(self.title);
		unsafe { igBegin(cs.as_ptr(), std::ptr::null_mut(), self.flags); }
	}
}

pub struct ImguiBuf {
	v: Vec<u8>,
}
impl ImguiBuf {
	pub fn new(size: usize) -> ImguiBuf {
		ImguiBuf {v: vec![0; size]}
	}
	pub fn len(&self) -> usize {
		self.v.len()
	}
	pub fn as_str(&self) -> String {
		let opt_len = self.v.iter()
			.enumerate()
			.filter_map(|(i,&x)|
				if x == 0 {
					Some(i)
				} else {
					None
				}
			).nth(0);
		let len = if let Some(n) = opt_len { n } else { self.v.len() };
		String::from_utf8(self.v[0..len].to_vec()).unwrap()
	}
	unsafe fn as_ptr(&mut self) -> *mut i8 {
		self.v.as_mut_ptr() as *mut i8
	}
}
