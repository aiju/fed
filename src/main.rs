mod gfx;

use gfx::*;

use std::collections::{HashMap, HashSet};

const POINT_RADIUS : f32 = 5.0;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct ID {
	id: u64
}
impl ID {
	fn new() -> ID {
		static mut I: u64 = 0;
		unsafe {
			I += 1;
			ID { id: I }
		}
	}
}

struct PointMan {
	pt: HashMap<ID, Vec2>,
}
impl PointMan {
	fn new() -> PointMan {
		PointMan {
			pt: HashMap::new()
		}
	}
	fn add(&mut self, p: Vec2) {
		self.pt.insert(ID::new(), p);
	}
	fn grab(&self, p: Vec2) -> Vec<ID> {
		self.pt.iter().filter(|(_,x)| x.dist(p) <= POINT_RADIUS).map(|(&id,_)| id).collect()
	}
	fn draw(&mut self, sel: &HashSet<ID>) -> Vec<DrawCmd> {
		self.pt.iter().map(|(id,&c)|
			DrawCmd::CircleFilled(c, POINT_RADIUS,
				if sel.contains(id) {
					Color::new(255, 127, 127, 255)
				} else {
					Color::new(127, 0, 0, 255)
				}
			)
		).collect()
	}
	fn mv(&mut self, id: ID, p: Vec2) {
		*self.pt.get_mut(&id).unwrap() = p;
	}
	fn mv_rel(&mut self, id: ID, p: Vec2) {
		*self.pt.get_mut(&id).unwrap() += p;
	}
}

#[derive(Copy, Clone, Debug)]
enum Constr {
	Hor(ID, ID),
	Ver(ID, ID),
	Dist(ID, ID, f32),
}
struct ConstrMan {
	ct: Vec<Constr>,
}
impl ConstrMan {
	fn new() -> ConstrMan {
		ConstrMan {
			ct: Vec::new(),
		}
	}
	fn add(&mut self, c: Constr) {
		self.ct.push(c);
	}
	fn solve(&mut self, pm: &mut PointMan) {
		if pm.pt.len() == 0 {
			return;
		}
		let mut newv: HashMap<ID, (Vec2, Vec2, Vec2)>
			= pm.pt.iter().map(|(&id, &x)| (id, (x,x,x))).collect();
		{
			for (_, t) in newv.iter_mut() {
				let v = t.2;
				t.1 = v;
			}
			for c in &self.ct {
				match c {
				Constr::Hor(a, b) =>
					{
						let ap = newv.get(a).unwrap().2;
						let bp = newv.get(b).unwrap().2;
						let r = horcons(ap, bp);
						(*newv.get_mut(a).unwrap()).2 = r.0;
						(*newv.get_mut(b).unwrap()).2 = r.1;
					},
				Constr::Ver(a, b) =>
					{
						let ap = newv.get(a).unwrap().2;
						let bp = newv.get(b).unwrap().2;
						let r = vercons(ap, bp);
						(*newv.get_mut(a).unwrap()).2 = r.0;
						(*newv.get_mut(b).unwrap()).2 = r.1;
					},
				Constr::Dist(a, b, d) =>
					{
						let ap = newv.get(a).unwrap().2;
						let bp = newv.get(b).unwrap().2;
						let r = distcons(ap, bp, *d);
						(*newv.get_mut(a).unwrap()).2 = r.0;
						(*newv.get_mut(b).unwrap()).2 = r.1;
					},
				}
			}
			let f = newv.iter().map(|(_, x)| x.1.dist(x.2)).fold(0./0., f32::max);
			if f < 1e-3 {
		//		break;
			}
		}
		for (&k, &(_, _, c)) in &newv {
			pm.mv(k, c)
		}
	}
}
fn horcons(a: Vec2, b: Vec2) -> (Vec2, Vec2) {
	(Vec2::new(a.x, a.y * 0.9 + b.y * 0.1),
	Vec2::new(b.x, b.y * 0.9 + a.y * 0.1))
} 
fn vercons(a: Vec2, b: Vec2) -> (Vec2, Vec2) {
	(Vec2::new(a.x * 0.9 + b.x * 0.1, a.y),
	Vec2::new(b.x * 0.9 + a.x * 0.1, b.y))
} 
fn distcons(a: Vec2, b: Vec2, d: f32) -> (Vec2, Vec2) {
	let cd = a.dist(b);
	let v = (a - b) * (d / cd - 1.0);
	(a + v * 0.1, b - v * 0.1)
} 


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
	downpos: Vec2,
	dim_buf: ImguiBuf,
}
impl FED {
	fn new() -> FED {
		FED {
			pm: PointMan::new(),
			cm: ConstrMan::new(),
			t: Tool::Move,
			sel: HashSet::new(),
			downpos: Vec2::zero(),
			dim_buf: ImguiBuf::new(512),
		}
	}
	fn moveclick(&mut self, p: Vec2, ctrl: bool) {
		if !ctrl {
			self.sel.clear();
		}
		for i in self.pm.grab(p) {
			self.sel.insert(i);
			break;
		}
		self.downpos = p;
	}
	fn movedown(&mut self, p: Vec2) {
		for &i in &self.sel {
			self.pm.mv_rel(i, p - self.downpos);
		}
		self.cm.solve(&mut self.pm);
		self.downpos = p;
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
					}
				},
			Tool::Add =>
				if imgui.is_mouse_clicked(0) {
					self.pm.add(p)
				},
			}
		}
		imgui.draw(&self.pm.draw(&self.sel), cp);
		imgui.end();
	}
}

fn main() {
	let mut fed = FED::new();
	gfx::init(&mut |imgui| fed.render(imgui));
}
