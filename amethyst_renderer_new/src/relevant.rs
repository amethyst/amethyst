
/// Type that can't be silently dropped.
/// If structure has field with type `Relevant`
/// than this structure can't be silently dropped either.
/// User has to deconstruct such types and call `Relevant::dispose`.
///
/// # Panics
///
/// Panics when dropped.
///
#[derive(Clone, Debug, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Relevant;

impl Relevant {
    /// Dispose this value.
    pub unsafe fn dispose(self) {
        ::std::mem::forget(self)
    }
}

impl Drop for Relevant {
    fn drop(&mut self) {
        panic!("This type can't be dropped")
    }
}
