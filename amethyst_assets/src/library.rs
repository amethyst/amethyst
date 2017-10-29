use Handle;

struct Library {
    id: Arc<u32>,
}

impl Library {
    pub fn handle<A, S>(s: S) -> Handle<A> {

    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Eq(bound = ""), Hash(bound = ""), PartialEq(bound = ""),
Debug(bound = ""))]
pub struct LibraryHandle<A: ?Sized> {}
