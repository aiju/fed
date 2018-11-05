extern crate sdl2;
extern crate gl;
extern crate imgui_sys;

use dat::*;
use hjimgui::*;

use self::imgui_sys::*;
use std::time::Instant;
use std::os::raw::*;
use self::gl::types::*;
use std::ffi::CString;

pub use self::sdl2::keyboard::Scancode;

pub struct GFX {
	event_pump: sdl2::EventPump,
	sdl: sdl2::Sdl,
	window: sdl2::video::Window,
	gl_context: sdl2::video::GLContext,
	startt: Instant,
}

impl GFX {
	pub fn new() -> GFX {
		let sdl = sdl2::init().unwrap();
		let video_subsystem = sdl.video().unwrap();
	
		let gl_attr = video_subsystem.gl_attr();
	
		gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
		gl_attr.set_context_version(4, 1);
	
		let window = video_subsystem
			.window("Game", 900, 900)
			.opengl()
			.build()
			.unwrap();
		
		let gl_context = window.gl_create_context().unwrap();
		let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
	
		window.gl_make_current(&gl_context).unwrap();
	
		let mut event_pump = sdl.event_pump().unwrap();
		
		GFX { event_pump, sdl, gl_context, window, startt: Instant::now() }
	}
	
	pub fn frame_start(&mut self, imgui: &mut Imgui) -> bool {
		self.startt = Instant::now();
		for event in self.event_pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit {..} => return false,
				sdl2::event::Event::TextInput { text, .. } =>
					imgui.add_text(&text),
				_ => ()
			}
		}
		let mouse = self.event_pump.mouse_state();
		imgui.mouse(mouse.x(), mouse.y(), mouse.left(), mouse.right());
		let kb = self.event_pump.keyboard_state();
		imgui.keyboard(kb, self.sdl.keyboard().mod_state());

		imgui.frame();
		true
	}
	
	pub fn frame_end(&mut self, imgui: &mut Imgui) {
		self.window.gl_make_current(&self.gl_context).unwrap();
		imgui.render();
		self.window.gl_swap_window();
		
		if let Some(diff) = std::time::Duration::from_nanos(1_000_000_000 / 60).checked_sub(self.startt.elapsed()) {
			std::thread::sleep(diff);
		}
	}
}
