

/// The type to use in associated constants for iterables
/// when simple array is not enough.
/// For recursive implementations for example.
#[derive(Derivative)]
#[derivative(Clone, Debug)]
pub enum ConstantList<T: 'static> {
    Node(T, &'static ConstantList<T>),
    End,
}

impl<T> Iterator for ConstantList<T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        use self::ConstantList::*;

        let next = match *self {
            End => return None,
            Node(_, next) => next.clone(),
        };

        match ::std::mem::replace(self, next) {
            End => unreachable!(),
            Node(item, _) => Some(item),
        }
    }
}


#[inline(always)]
pub fn is_slice_sorted<T>(slice: &[T]) -> bool
where
    T: Ord,
{
    if slice.len() == 0 {
        return true;
    }
    for i in 0..slice.len() - 1 {
        if slice[i] > slice[i + 1] {
            return false;
        }
    }
    return true;
}

#[inline(always)]
pub fn is_slice_sorted_by_key<T, K, F>(slice: &[T], mut f: F) -> bool
where
    K: Ord,
    F: FnMut(&T) -> K,
{
    if slice.len() == 0 {
        return true;
    }
    for i in 0..slice.len() - 1 {
        if f(&slice[i]) > f(&slice[i + 1]) {
            return false;
        }
    }
    return true;
}
