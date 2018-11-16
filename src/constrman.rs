use dat::*;
use pointman::*;
use hjimgui::*;

use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug)]
pub enum Constr {
	Hor(ID, ID),
	Ver(ID, ID),
	Dist(ID, ID, f32),
}
pub struct ConstrMan {
	ct: Vec<Constr>,
}
impl ConstrMan {
	pub fn new() -> ConstrMan {
		ConstrMan {
			ct: Vec::new(),
		}
	}
	pub fn add(&mut self, c: Constr) {
		self.ct.push(c);
	}
	pub fn solve(&mut self, pm: &mut PointMan) {
		let mut newv: HashMap<ID, (Vec2, Vec2, Vec2)>
			= pm.iter().map(|(id, &x)| (id, (x,x,x))).collect();
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
			pm[k] = c
		}
	}
	pub fn draw(&mut self, pm: &PointMan) -> Vec<DrawCmd> {
		let grey = Color::new(128, 128, 128, 255);
		self.ct.iter().map(|x|
			match *x {
			Constr::Hor(a, b) => DrawCmd::Line(pm[a], pm[b], grey, 1.0),
			Constr::Ver(a, b) => DrawCmd::Line(pm[a], pm[b], grey, 1.0),
			Constr::Dist(a, b, _) => DrawCmd::Line(pm[a], pm[b], grey, 1.0)
			}).collect()
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
