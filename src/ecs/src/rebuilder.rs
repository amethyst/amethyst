/// Rebuilder is a function (or a wrapper of that function) that takes an immutable reference to a component T,
/// a list of immutable references to required components (could be empty), and a mutable reference to memory for a new component T
/// which is used to write a updated component to.
use std::any::{Any, TypeId};

pub struct Rebuilder {
	component_type: TypeId,
	arguments_type: TypeId,
	func: Box<Any> //TODO: make that a box of FnOnce
	//Fn(&Any, Any, &mut Any)
}

impl Rebuilder {
	pub fn new<T: Any, Args: Any, Func>(function: Func) -> Rebuilder
		where Func: Fn(&T, &Args, &mut T) + Any
		{
		Rebuilder {
			component_type: TypeId::of::<T>(),
			arguments_type: TypeId::of::<Args>(),
			func: Box::<Fn(&T, &Args, &mut T)>::new(function)
		}
	}
	
	pub fn rebuild<T: Any, Args: Any>(&self, current: &T, args: &Args, future: &mut T) {
		assert_eq!(self.component_type, TypeId::of::<T>());
		assert_eq!(self.arguments_type, TypeId::of::<Args>());
		// We cast Box<Any> to Any and then to Box<Fn(..)> and use it
		//let b: &Any = &self.func;
		//b.downcast_ref::<Box<Fn(&T, &Args, &mut T)>>().unwrap()(current, args, future);
		let b: Box<Fn(&T, &Args, &mut T)> = self.func.downcast().unwrap();
		b(current, args, future);
	}
}
