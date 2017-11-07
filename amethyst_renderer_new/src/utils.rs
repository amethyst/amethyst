

#[inline(always)]
pub fn is_slice_sorted<T>(slice: &[T]) -> bool
where
    T: Clone + Ord,
{
    let mut clone = slice.to_vec();
    clone.sort();
    &clone[..] == slice
}

#[inline(always)]
pub fn is_slice_sorted_by_key<T, K, F>(slice: &[T], f: F) -> bool
where
    K: Clone + Ord,
    F: FnMut(&T) -> K,
{
    is_slice_sorted(&slice.iter().map(f).collect::<Vec<K>>())
}
