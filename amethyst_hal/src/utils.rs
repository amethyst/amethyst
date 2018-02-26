use std::borrow::Cow;
use hal::memory::{Pod, cast_slice};

pub fn is_slice_sorted<T: Ord>(slice: &[T]) -> bool {
    is_slice_sorted_by_key(slice, |i| i)
}

pub fn is_slice_sorted_by_key<'a, T, K: Ord, F: Fn(&'a T) -> K>(slice: &'a [T], f: F) -> bool {
    if let Some((first, slice)) = slice.split_first() {
        let mut cmp = f(first);
        for item in slice {
            let item = f(item);
            if cmp > item {
                return false;
            }
            cmp = item;
        }
    }
    true
}

pub fn cast_vec<A: Pod, B: Pod>(mut vec: Vec<A>) -> Vec<B> {
    use std::mem;

    let raw_len = mem::size_of::<A>() * vec.len();
    let len = raw_len / mem::size_of::<B>();

    let raw_cap = mem::size_of::<A>() * vec.capacity();
    let cap = raw_cap / mem::size_of::<B>();
    assert_eq!(raw_cap, mem::size_of::<B>() * cap);

    let ptr = vec.as_mut_ptr();
    mem::forget(vec);
    unsafe {
        Vec::from_raw_parts(ptr as _, len, cap)
    }
}

pub fn cast_cow<A: Pod, B: Pod>(cow: Cow<[A]>) -> Cow<[B]> {
    match cow {
        Cow::Borrowed(slice) => Cow::Borrowed(cast_slice(slice)),
        Cow::Owned(vec) => Cow::Owned(cast_vec(vec)),
    }
}
