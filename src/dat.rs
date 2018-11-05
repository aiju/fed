extern crate imgui_sys;
use self::imgui_sys::ImVec2;

#[derive(Copy,Clone,Debug)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32,
}
impl Vec2 {
	pub fn new(x: f32, y: f32) -> Vec2 {
		Vec2 {x, y}
	}
	pub fn zero() -> Vec2 {
		Vec2 {x: 0.0, y: 0.0}
	}
	pub fn imvec(&self) -> ImVec2 {
		ImVec2 { x: self.x, y: self.y }
	}
	pub fn dist(&self, v: Vec2) -> f32 {
		(self.x - v.x).hypot(self.y - v.y)
	}
}
impl std::ops::Add for Vec2 {
	type Output = Vec2;
	fn add(self, other: Vec2) -> Vec2 {
		Vec2 {
			x: self.x + other.x,
			y: self.y + other.y,
		}
	}
}
impl std::ops::Sub for Vec2 {
	type Output = Vec2;
	fn sub(self, other: Vec2) -> Vec2 {
		Vec2 {
			x: self.x - other.x,
			y: self.y - other.y,
		}
	}
}
impl std::ops::Mul<f32> for Vec2 {
	type Output = Vec2;
	fn mul(self, other: f32) -> Vec2 {
		Vec2 {
			x: self.x * other,
			y: self.y * other,
		}
	}
}
impl std::ops::AddAssign for Vec2 {
	fn add_assign(&mut self, other: Vec2) {
		self.x += other.x;
		self.y += other.y;
	}
}

#[derive(Clone,Copy,Debug)]
pub struct Color {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}
impl Color {
	pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
		Color{ r, g, b, a }
	}
	pub fn to_u32(&self) -> u32 {
		(self.a as u32) << 24 | (self.b as u32) << 16 | (self.g as u32) << 8 | (self.r as u32)
	}
}
