extern crate imgui_sys;
extern crate gl;
extern crate sdl2;

mod gfx;
mod dat;
mod hjgl;
mod hjimgui;
mod constr;

use gfx::*;
use dat::*;
use hjimgui::*;
use constr::*;

use std::collections::{HashMap, HashSet};

const POINT_RADIUS : f32 = 5.0;

type Points = IDMap<Vec2>; 

fn pointgrab(l: &Points, p: Vec2) -> Vec<ID> {
	l.iter().filter(|(_,x)| x.dist(p) <= POINT_RADIUS).map(|(id,_)| id).collect()
}

fn pointdraw(l: &Points, sel: &HashSet<ID>) -> Vec<DrawCmd> {
	l.iter().map(|(id,&c)|
		DrawCmd::CircleFilled(c, POINT_RADIUS,
			if sel.contains(&id) {
				Color::new(255, 127, 127, 255)
			} else {
				Color::new(127, 0, 0, 255)
			}
		)
	).collect()
}

#[derive(Debug,PartialEq)]
enum Tool {
	Move,
	Add
}

struct FED {
	points: Points,
	constrs: Constrs,
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
			points: Points::new(),
			constrs: Vec::new(),
			t: Tool::Move,
			sel: HashSet::new(),
			startpos: Vec2::zero(),
			downpos: Vec2::zero(),
			dim_buf: ImguiBuf::new(512),
			rectsel: false,
		}
	}
	fn moveclick(&mut self, p: Vec2, ctrl: bool) {
		let g = pointgrab(&self.points, p);
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
				self.points[i] += p - self.downpos;
			}
		}
		self.downpos = p;
	}
	fn moveup(&mut self, p: Vec2) {
		if self.rectsel {
			let minx = if p.x < self.startpos.x { p.x } else { self.startpos.x };
			let maxx = if p.x > self.startpos.x { p.x } else { self.startpos.x };
			let miny = if p.y < self.startpos.y { p.y } else { self.startpos.y };
			let maxy = if p.y > self.startpos.y { p.y } else { self.startpos.y };
			self.sel = self.points.iter().filter(|&(_, &q)| q.x >= minx && q.x <= maxx && q.y >= miny && q.y <= maxy).map(|(id,_)| id).collect();
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
						self.constrs.push(Constr::Hor(a, b));
					}
				}
			}
		}
		if imgui.button("Vertical", Vec2::zero()) {
			for &a in &self.sel {
				for &b in &self.sel {
					if a != b {
						self.constrs.push(Constr::Ver(a, b));
					}
				}
			}
		}
		imgui.input_text("Dim", &mut self.dim_buf);
		if imgui.button("Dimension", Vec2::zero()) {
			if let Ok(d) = self.dim_buf.as_str().parse::<f32>() {
				for &a in &self.sel {
					for &b in &self.sel {
						if a != b {
							self.constrs.push(Constr::Dist(a, b, d));
						}
					}
				}
			}
		}
		
		let cp = imgui.cursor_screen_pos();
		imgui.invisible_button("canvas", Vec2::new(600.0, 600.0));
		imgui.draw(&[DrawCmd::RectFilled(cp, cp + Vec2::new(600.0, 600.0), Color::new(255, 255, 255, 255))], Vec2::zero());
		let p = imgui.mouse_pos() - cp;
		if imgui.is_item_hovered() {
                       match self.t {
			Tool::Move => {
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
					self.points.insert(ID::new(), p)
				}
			}
		}
		imgui.draw(&pointdraw(&self.points, &self.sel), cp);
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
