extern crate imgui_sys;
extern crate gl;
extern crate sdl2;

mod gfx;
mod dat;
mod hjgl;
mod hjimgui;
mod pointman;
mod constrman;

use gfx::*;
use dat::*;
use hjimgui::*;
use pointman::*;
use constrman::*;

use std::collections::{HashMap, HashSet};

#[derive(PartialEq)]
enum Tool {
	Move,
	Add,
}

struct FED {
	pm: PointMan,
	cm: ConstrMan,
	t: Tool,
	sel: HashSet<ID>,
	startpos: Vec2,
	downpos: Vec2,
	dim_buf: ImguiBuf,
	rectsel: bool
}
impl FED {
	fn new() -> FED {
		FED {
			pm: PointMan::new(),
			cm: ConstrMan::new(),
			t: Tool::Move,
			sel: HashSet::new(),
			startpos: Vec2::zero(),
			downpos: Vec2::zero(),
			dim_buf: ImguiBuf::new(512),
			rectsel: false,
		}
	}
	fn moveclick(&mut self, p: Vec2, ctrl: bool) {
		let g = self.pm.grab(p);
		let sel_clicked = g.iter().fold(true, |a, x| a&&self.sel.contains(x));
		if !ctrl && !sel_clicked {
			self.sel.clear();
		}
		if g.len() == 0 {
			self.rectsel = true;
		} else {
			for i in g {
				self.sel.insert(i);
				break;
			}
		}
		self.startpos = p;
		self.downpos = p;
	}
	fn movedown(&mut self, p: Vec2) {
		if self.rectsel {
		} else {
			for &i in &self.sel {
				self.pm[i] += p - self.downpos;
			}
			self.cm.solve(&mut self.pm);
		}
		self.downpos = p;
	}
	fn moveup(&mut self, p: Vec2) {
		if self.rectsel {
			let minx = if p.x < self.startpos.x { p.x } else { self.startpos.x };
			let maxx = if p.x > self.startpos.x { p.x } else { self.startpos.x };
			let miny = if p.y < self.startpos.y { p.y } else { self.startpos.y };
			let maxy = if p.y > self.startpos.y { p.y } else { self.startpos.y };
			self.sel = self.pm.iter().filter(|&(_, &q)| q.x >= minx && q.x <= maxx && q.y >= miny && q.y <= maxy).map(|(id,_)| id).collect();
			self.rectsel = false;
		}
	}
	fn render(&mut self, imgui: &mut Imgui) {
		imgui.window("Derp")
			.pos(100.0, 100.0)
			.begin();
		
		if imgui.radio_button("Move", self.t == Tool::Move) {
			self.t = Tool::Move;
		}
		if imgui.radio_button("Add", self.t == Tool::Add) {
			self.t = Tool::Add;
		}
		if imgui.button("Horizontal", Vec2::zero()) {
			for &a in &self.sel {
				for &b in &self.sel {
					if a != b {
						self.cm.add(Constr::Hor(a, b));
					}
				}
			}
			self.cm.solve(&mut self.pm)
		}
		if imgui.button("Vertical", Vec2::zero()) {
			for &a in &self.sel {
				for &b in &self.sel {
					if a != b {
						self.cm.add(Constr::Ver(a, b));
					}
				}
			}
			self.cm.solve(&mut self.pm)
		}
		imgui.input_text("Dim", &mut self.dim_buf);
		if imgui.button("Dimension", Vec2::zero()) {
			if let Ok(d) = self.dim_buf.as_str().parse::<f32>() {
				for &a in &self.sel {
					for &b in &self.sel {
						if a != b {
							self.cm.add(Constr::Dist(a, b, d));
						}
					}
				}
				self.cm.solve(&mut self.pm);
			}
		}
		self.cm.solve(&mut self.pm);
		
		let cp = imgui.cursor_screen_pos();
		imgui.invisible_button("canvas", Vec2::new(600.0, 600.0));
		imgui.draw(&[DrawCmd::RectFilled(cp, cp + Vec2::new(600.0, 600.0), Color::new(255, 255, 255, 255))], Vec2::zero());
		let p = imgui.mouse_pos() - cp;
		if imgui.is_item_hovered() {
			match self.t {
			Tool::Move =>
				{
					if imgui.is_mouse_clicked(0) {
						self.moveclick(p, imgui.is_ctrl_down());
					} else if imgui.is_mouse_down(0) {
						self.movedown(p);
					} else if imgui.is_mouse_released(0) {
						self.moveup(p);
					}
				},
			Tool::Add =>
				if imgui.is_mouse_clicked(0) {
					self.pm.add(p)
				},
			}
		}
		imgui.draw(&self.pm.draw(&self.sel), cp);
		imgui.draw(&self.cm.draw(&self.pm), cp);
		if self.rectsel {
			imgui.draw(&[DrawCmd::Rect(self.startpos, self.downpos, Color::new(0, 0, 0, 255), 1.0)], cp);
		}
		imgui.end();
	}
}

fn main() {
	let mut fed = FED::new();
	let mut gfx = GFX::new();
	let mut imgui = Imgui::new(900.0, 900.0);
	
	while gfx.frame_start(&mut imgui) {
		fed.render(&mut imgui);
		gfx.frame_end(&mut imgui);
	}
}
