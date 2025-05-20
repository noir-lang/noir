use crate::utils::arrays::find_index_hint;

// Copied from the stdlib Array.sort_via.
// But this one doesn't use `ordering` to check that the array is sorted.
// This is to allow preserving the order of equal values.
pub unconstrained fn sort_by<T, let N: u32, Env>(
    array: [T; N],
    ordering: fn[Env](T, T) -> bool,
) -> [T; N] {
    let mut result = array;
    let sorted_index = get_sorting_index(array, ordering);
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

/// Returns the index of the elements in the array that would sort it, using the provided custom sorting function.
unconstrained fn get_sorting_index<T, let N: u32, Env>(
    array: [T; N],
    ordering: fn[Env](T, T) -> bool,
) -> [u32; N] {
    let mut result = [0; N];
    let mut a = array;
    for i in 0..N {
        result[i] = i;
    }
    for i in 1..N {
        for j in 0..i {
            if ordering(a[i], a[j]) {
                let old_a_j = a[j];
                a[j] = a[i];
                a[i] = old_a_j;
                let old_j = result[j];
                result[j] = result[i];
                result[i] = old_j;
            }
        }
    }
    result
}
