/// `Send + Sync` wrapper for objects that may not be `Send` or `Sync`.
///
/// # Panics
///
/// Panics if the inner value is accessed from a thread that is not the one that created the
/// wrapper.
#[derive(Debug)]
pub struct Ss<T> {
    value: T,
    thread: std::thread::ThreadId,
}

impl<T> Ss<T> {
    /// Returns a new `Send + Sync` wrapper.
    pub fn new(value: T) -> Self {
        Ss {
            value,
            thread: std::thread::current().id(),
        }
    }

    /// Returns an immutable reference to the inner value.
    pub fn get_ref(st: &Self) -> &T {
        assert_eq!(std::thread::current().id(), st.thread);
        &st.value
    }

    /// Returns a mutable reference to the inner value.
    pub fn get_mut(st: &mut Self) -> &mut T {
        assert_eq!(std::thread::current().id(), st.thread);
        &mut st.value
    }

    /// Returns the inner value.
    pub fn into_inner(st: Self) -> T {
        assert_eq!(std::thread::current().id(), st.thread);
        st.value
    }
}

impl<T> std::ops::Deref for Ss<T> {
    type Target = T;

    fn deref(&self) -> &T {
        Self::get_ref(self)
    }
}

impl<T> std::ops::DerefMut for Ss<T> {
    fn deref_mut(&mut self) -> &mut T {
        Self::get_mut(self)
    }
}

unsafe impl<T> Send for Ss<T> {}
unsafe impl<T> Sync for Ss<T> {}
