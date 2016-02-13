//! A runtime-typed vector. All DynVecs are of the same type, regardless of their elements' types.

use std::any::{Any, TypeId};
use std::mem::{transmute, size_of};
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct DynVec {
    vec: Vec<u8>,
    unused: Vec<usize>,
    t: TypeId,
    size: usize,
}

impl DynVec {
    /// Creates a new dynamically typed vector of type T
    pub fn new<T: Any>() -> DynVec {
        assert!(size_of::<T>() > 0);
        DynVec {
            vec: Vec::new(),
            unused: Vec::new(),
            t: TypeId::of::<T>(),
            size: size_of::<T>(),
        }
    }

    /// Returns a ref to ith component in the vector
    /// # Panics
    /// Panics if the type T does not match with the vector's type
    pub fn get_component<T: Any>(&self, i: usize) -> Option<&T> {
        unsafe {
            assert_eq!(self.t, TypeId::of::<T>()); //TODO: replace with Option or Result?
            Some(transmute::<&u8, &T>(self.vec.index(i * self.size)))
        }
        // TODO: check bounds
    }

    /// Returns a mutable ref to ith component in the vector
    /// # Panics
    /// Panics if the type T does not match with the vector's type
    pub fn get_component_mut<T: Any>(&mut self, i: usize) -> Option<&mut T> {
        unsafe {
            assert_eq!(self.t, TypeId::of::<T>()); //TODO: replace with Option or Result?
            Some(transmute::<&mut u8, &mut T>(self.vec.index_mut(i * self.size)))
        }
        // TODO: check bounds
    }

    /// Adds a new element and returns its index
    /// # Panics
    /// Panics if the type T does not match with the vector's type
    pub fn add<T: Any>(&mut self, val: T) -> usize {
        unsafe {
            use std::slice::from_raw_parts;
            assert_eq!(self.t, TypeId::of::<T>()); //TODO: replace with Option or Result?
            let slice: &[u8] = from_raw_parts::<u8>(transmute(&val), self.size);
            if let Some(index) = self.unused.pop() {
                for i in 0..self.size {
                    self.vec[i + index * self.size] = slice[i]; //TODO: replace with memcpy
                }
                index
            } else {
                self.vec.extend_from_slice(slice);
                self.vec.len() / self.size - 1
            }
        }
    }

    /// Removes an element
    pub fn remove(&mut self, index: usize) {
        assert!(index * self.size < self.vec.len());
        assert!(!self.unused.contains(&index));
        self.unused.push(index);
    }
}


// Unit tests
#[cfg(test)]
mod tests {
    use super::DynVec;

    #[test]
    fn add_remove() {
        struct Test(u32, f64);
        let mut vec = DynVec::new::<Test>();
        let id1 = vec.add(Test(32, 32.0));
        let id2 = vec.add(Test(35, 64.125));
        assert!(id1 != id2);
        {
            println!("{} {}", id1, id2);
            let ref1 = vec.get_component::<Test>(id1).unwrap();
            let ref2 = vec.get_component::<Test>(id2).unwrap();
            assert_eq!(ref1.0, 32);
            assert_eq!(ref1.1, 32.0); // Checking floats for equality? Anyway, it should work in this case.
            assert_eq!(ref2.0, 35);
            assert_eq!(ref2.1, 64.125); // Same
        }
        let id3 = vec.add(Test(0, 0.0));
        vec.remove(id2);
        let id4 = vec.add(Test(1, 0.25));
        assert_eq!(id2, id4);
        {
            let ref4 = vec.get_component::<Test>(id4).unwrap();
            assert_eq!(ref4.0, 1);
            assert_eq!(ref4.1, 0.25); // Same
        }
    }

    #[test]
    #[should_panic]
    fn type_check() {
        struct Struct1(u8);
        struct Struct2(u8);
        let mut vec = DynVec::new::<Struct1>();
        vec.add(Struct2(0));
    }

    #[test]
    #[should_panic]
    fn zero_sized_structs() {
        let vec = DynVec::new::<()>();
    }
}
