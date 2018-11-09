use dat::*;
use hjimgui::*;
use std::collections::HashSet;
use std::ops::*;

const POINT_RADIUS : f32 = 5.0;

pub struct PointMan {
	pt: IDMap<Vec2>
}
impl PointMan {
	pub fn new() -> PointMan {
		PointMan {
			pt: IDMap::new()
		}
	}
	pub fn add(&mut self, p: Vec2) {
		self.pt.insert(ID::new(), p);
	}
	pub fn grab(&self, p: Vec2) -> Vec<ID> {
		self.pt.iter().filter(|(_,x)| x.dist(p) <= POINT_RADIUS).map(|(id,_)| id).collect()
	}
	pub fn draw(&mut self, sel: &HashSet<ID>) -> Vec<DrawCmd> {
		self.pt.iter().map(|(id,&c)|
			DrawCmd::CircleFilled(c, POINT_RADIUS,
				if sel.contains(&id) {
					Color::new(255, 127, 127, 255)
				} else {
					Color::new(127, 0, 0, 255)
				}
			)
		).collect()
	}
	pub fn iter(&self) -> IDMapIterator<Vec2> {
		self.pt.iter()
	}
}
impl Index<ID> for PointMan {
	type Output = Vec2;
	fn index(&self, id: ID) -> &Vec2 {
		return &self.pt[id]
	}
}
impl IndexMut<ID> for PointMan {
	fn index_mut(&mut self, id: ID) -> &mut Vec2 {
		return &mut self.pt[id]
	}
}
