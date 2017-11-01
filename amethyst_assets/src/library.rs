use std::marker::PhantomData;
use std::sync::{Arc, Weak};

use fnv::FnvHashMap;
use hibitset::BitSet;
use specs::{UnprotectedStorage, VecStorage};

use storage::HandleAlloc;
use Handle;

struct Library {
    id: Arc<u32>,
}

impl Library {
    pub fn handle<A, S>(s: S) -> Handle<A> {
        unimplemented!()
    }
}

pub struct LibraryData<A> {
    // TODO: reconsider `String`
    map: FnvHashMap<String, HandleAlloc<A>>,
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Eq(bound = ""), Hash(bound = ""), PartialEq(bound = ""),
Debug(bound = ""))]
pub struct LibraryHandle<A> {
    id: Arc<u32>,
    key: String,
    marker: PhantomData<A>,
}

pub struct WeakLibraryHandle<A> {
    id: Weak<u32>,
    key: String,
    marker: PhantomData<A>,
}

pub struct LibraryStorage<A> {
    bitset: BitSet,
    libs: VecStorage<LibraryData<A>>,
}

impl<A> LibraryStorage<A> {
    pub fn get(&self, h: LibraryHandle<A>) -> Option<&HandleAlloc<A>> {
        let id = *h.id.as_ref();

        if self.bitset.contains(id) {
            unsafe {
                self.libs.get(id).map.get(&h.key)
            }
        } else {
            None
        }
    }
}
