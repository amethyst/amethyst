

#[inline(always)]
pub fn is_slice_sorted<T>(slice: &[T]) -> bool
where
    T: Ord,
{
    for i in 0..slice.len() - 2 {
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
    for i in 0..slice.len() - 2 {
        if f(&slice[i]) > f(&slice[i + 1]) {
            return false;
        }
    }
    return true;
}
