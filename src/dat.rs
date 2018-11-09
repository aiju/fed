extern crate imgui_sys;
use self::imgui_sys::ImVec2;
use std::ops::*;

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

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ID {
	slot: u32,
	gen: u32
}
impl ID {
	pub fn new() -> ID {
		static mut I: u32 = 0;
		unsafe {
			I += 1;
			ID { slot: I, gen: 0 }
		}
	}
	pub fn slot(&self) -> u32 {
		self.slot
	}
	pub fn gen(&self) -> u32 {
		self.gen
	}
}

pub struct IDMap<T> {
	data: Vec<Option<(u32,T)>>
}
impl<T> IDMap<T> {
	pub fn new() -> IDMap<T> {
		IDMap { data: Vec::new() }
	}
	pub fn insert(&mut self, key: ID, val: T) {
		if key.slot() as usize >= self.data.len() {
			let n = key.slot() as usize + 1 - self.data.len();
			self.data.reserve(n);
			for i in 0..n {
				self.data.push(None)
			}
		}
		let p = &mut self.data[key.slot() as usize];
		if let Some((gen, _)) = *p {
			assert!(gen == key.gen());
		}
		*p = Some((key.gen(), val));
	}
	pub fn get(&self, key: ID) -> Option<&T> {
		if key.slot() as usize >= self.data.len() {
			return None;
		}
		match self.data[key.slot() as usize] {
		Some((s, ref val)) if s == key.gen() => Some(val),
		_ => None,
		}
	}
	pub fn get_mut(&mut self, key: ID) -> Option<&mut T> {
		if key.slot() as usize >= self.data.len() {
			return None;
		}
		match self.data[key.slot() as usize] {
		Some((s, ref mut val)) if s == key.gen() => Some(val),
		_ => None,
		}
	}
}
impl<'a, T> IDMap<T> {
	pub fn iter(&'a self) -> IDMapIterator<'a, T> {
		IDMapIterator { map: self, id: 0 }
	}
}
impl<T> Index<ID> for IDMap<T> {
	type Output = T;
	
	fn index(&self, key: ID) -> &T {
		self.get(key).unwrap()
	}
}
impl<T> IndexMut<ID> for IDMap<T> {
	fn index_mut(&mut self, key: ID) -> &mut T {
		self.get_mut(key).unwrap()
	}
}
pub struct IDMapIterator<'a, T: 'a> {
	map: &'a IDMap<T>,
	id: u32
}
impl<'a, T> Iterator for IDMapIterator<'a, T> {
	type Item = (ID, &'a T);
	
	fn next(&mut self) -> Option<(ID, &'a T)> {
		loop {
			if self.id as usize >= self.map.data.len() {
				return None
			}
			let p = &self.map.data[self.id as usize];
			let slot = self.id;
			self.id += 1;
			if let Some((gen, ref val)) = *p {
				return Some((ID{slot, gen}, val))
			}
		}
	}
}
