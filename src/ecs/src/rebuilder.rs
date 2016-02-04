/// Rebuilder is a function (or a wrapper of that function) that takes an immutable reference to a component T,
/// a list of immutable references to required components (could be empty), and a mutable reference to memory for a new component T
/// which is used to write a updated component to.
use std::any::{Any, TypeId};
use std::mem::transmute;

pub struct Rebuilder {
    component_type: TypeId,
    arguments_type: TypeId,
    func: Box<Any>,
}

impl Rebuilder {
    pub fn new<T: Any, Args: Any, Func>(function: Func) -> Rebuilder
        where Func: Fn(&T, &Args, &mut T) + Any
    {
        Rebuilder {
            component_type: TypeId::of::<T>(),
            arguments_type: TypeId::of::<Args>(),
            func: unsafe { transmute::<Box<Fn(&T, &Args, &mut T)>, Box<Any>>(Box::new(function)) }, /* Cast Box<Fn> to Box<Any> */
        }
    }

    pub fn rebuild<T: Any, Args: Any>(&self, current: &T, args: &Args, future: &mut T) {
        assert_eq!(self.component_type, TypeId::of::<T>());
        assert_eq!(self.arguments_type, TypeId::of::<Args>());
        let b: &Box<Fn(&T, &Args, &mut T)> = unsafe {
            transmute::<&Box<Any>, &Box<Fn(&T, &Args, &mut T)>>(&self.func)
        }; // Cast Box<Any> to Box<Fn>
        b(current, args, future);
    }
}
