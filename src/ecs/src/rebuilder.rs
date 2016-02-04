/// Rebuilder is a function (or a wrapper of that function) that takes an immutable reference to a component T,
/// a list of immutable references to required components (could be empty), and a mutable reference to memory for a new component T
/// which is used to write a updated component to.
use std::any::{Any, TypeId};

pub struct Rebuilder {
	component_type: TypeId,
	required_types: Vec<TypeId>,
	func: Box<u8> //TODO: make that a box of FnOnce
}

impl Rebuilder {
	pub fn new<T: Any>(required: &[TypeId]) -> Rebuilder {
		Rebuilder {
			component_type: TypeId::of::<T>(),
			required_types: {
				let mut v = Vec::with_capacity(required.len());
				v.extend_from_slice(required);
				v
			},
			func: Box::new(0)
		}
	}
	
	pub fn rebuild<T: Any>(&self, current: &T, required: Vec<&Any>, future: &mut T) {
		assert!(required.len() >= self.required_types.len());
		for i in 0..self.required_types.len() {
			//Sadly, Any::get_type_id() is unstable
			//TODO: Find another way to do runtime type checks
			//assert_eq!(required[i].get_type_id, self.required_types[i]);
		}
	}
}
