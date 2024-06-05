// Swap the items so that they are not ordered by counters.
pub fn swap_items<T, N>(vec: &mut BoundedVec<T, N>, from_index: u64, to_index: u64) {
    let tmp = vec.storage[from_index];
    vec.storage[from_index] = vec.storage[to_index];
    vec.storage[to_index] = tmp;
}
