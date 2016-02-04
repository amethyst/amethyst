use std::any::{Any,TypeId};
use std::mem::{transmute, size_of};
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct DynVec {
	vec: Vec<u8>,
	t: TypeId,
	size: usize
}

impl DynVec {
	pub fn new<T: 'static + Any>() -> DynVec {
		DynVec {
			vec: vec![],
			t: TypeId::of::<T>(),
			size: size_of::<T>()
		}
	}
	
	pub fn get_component<T: 'static + Any>(&self, i: usize) -> &T {
		unsafe {
			assert_eq!(self.t, TypeId::of::<T>()); //TODO: replace with Option or Result?
			transmute::<&u8, &T>(self.vec.index(i * self.size))
		}
		//TODO: check bounds
	}
	
	pub fn get_component_mut<T: 'static + Any>(&mut self, i: usize) -> &mut T {
		unsafe {
			assert_eq!(self.t, TypeId::of::<T>()); //TODO: replace with Option or Result?
			transmute::<&mut u8, &mut T>(self.vec.index_mut(i * self.size))
		}
		//TODO: check bounds
	}
	
	pub fn push<T: 'static + Any>(&mut self, val: T) {
		unsafe {
			use std::slice::from_raw_parts;
			assert_eq!(self.t, TypeId::of::<T>()); //TODO: replace with Option or Result?
			let slice: &[u8] = from_raw_parts::<u8>(transmute(&val), self.size);
			self.vec.extend_from_slice(slice);
		}
	}
}
