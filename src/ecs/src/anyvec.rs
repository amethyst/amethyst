use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::slice;
use std::mem;
use std::ptr;

#[derive(Debug)]
pub struct AnyVec {
    data: Vec<u8>,
    meta: Vec<Meta>,
    type_sizes: HashMap<TypeId, usize>,
}

#[derive(Debug)]
struct Meta {
    data_index: usize,
    type_id: TypeId,
}

impl AnyVec {
    pub fn new() -> AnyVec {
        AnyVec {
            data: Vec::new(),
            meta: Vec::new(),
            type_sizes: HashMap::new(),
        }
    }
    
    pub fn push<T: Any>(&mut self, value: T) {
        let index = self.meta.len();
        self.insert(index, value);
    }
    
    pub fn insert<T: Any>(&mut self, index: usize, element: T) {
        let type_id = TypeId::of::<T>();
        let type_size = mem::size_of::<T>();
        
        let s: &[u8] = unsafe { 
            slice::from_raw_parts(&element as *const _ as *const u8, type_size)
        };
        
        let data_index: usize = match self.meta.get(index) {
            Some(meta) => meta.data_index,
            None => self.data.len(),
        };
        
        self.meta.insert(index, Meta { data_index: data_index, type_id: type_id });
        for i in (index + 1)..self.meta.len() {
            self.meta[i].data_index += type_size;
        }
        
        if !self.type_sizes.contains_key(&type_id) {
            self.type_sizes.insert(type_id, type_size);
        }
        
        for i in 0..s.len() {
            self.data.insert(data_index + i, s[i]);
        }
    }
    
    pub fn get<T: Any>(&self, index: usize) -> &T {
        let meta = &self.meta[index];
        assert_eq!(meta.type_id, TypeId::of::<T>());
        unsafe {
            ptr::read(&&self.data[meta.data_index] as *const _ as *const &T)
        }
    }
    
    pub fn get_mut<T: Any>(&self, index: usize) -> &mut T {
        let meta = &self.meta[index];
        assert_eq!(meta.type_id, TypeId::of::<T>());
        unsafe {
            ptr::read(&&self.data[meta.data_index] as *const _ as *const &mut T)
        }
    }
}

#[derive(Debug)]
struct Position {
    x: f64,
    y: f64,
}

#[test]
fn main() {
    let mut any_vec = AnyVec::new();
    
    any_vec.push(99 as u16);
    any_vec.push(Position { x: 0.0, y: 0.0});
    any_vec.push("Test");
    
    println!("{}", any_vec.get::<u16>(0));
    println!("{:?}", any_vec.get::<Position>(1));
    
    any_vec.get_mut::<Position>(1).x = 1.0;
    println!("{:?}", any_vec.get::<Position>(1));
    
    println!("{}", any_vec.get::<&str>(2));
}
