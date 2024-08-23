use crate::utils::arrays::find_index_hint;

// Copied from the stdlib Array.sort_via, but this one doesn't use `ordering` to check that the array is sorted.
pub fn sort_by<T, let N: u32, Env>(array: [T; N], ordering: fn[Env](T, T) -> bool) -> [T; N] {
    let mut result = array;
    let sorted_index = array.get_sorting_index(ordering);
    // Ensure the indexes are correct
    for i in 0..N {
        let pos = find_index_hint(sorted_index, |index: u32| index == i);
        assert(sorted_index[pos] == i);
    }
    // Sort the array using the indexes
    for i in 0..N {
        result[i] = array[sorted_index[i]];
    }

    // Ensure the array is sorted
    // for i in 0..N - 1 {
    //     assert(ordering(result[i], result[i + 1]));
    // }

    result
}
