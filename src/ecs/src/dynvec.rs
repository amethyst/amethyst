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
                self.vec.len() - 1
            }
        }
    }

    pub fn remove<T: Any>(&mut self, index: usize) {
        assert!(index * self.size < self.vec.len());
        assert!(!self.unused.contains(&index));
        self.unused.push(index);
    }
}
