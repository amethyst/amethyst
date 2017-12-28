
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
