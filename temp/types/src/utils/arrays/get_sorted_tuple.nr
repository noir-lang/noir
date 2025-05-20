use crate::utils::arrays::sort_by::sort_by;

pub struct SortedTuple<T> {
    pub elem: T,
    pub original_index: u32,
}

pub unconstrained fn get_sorted_tuple<T, let N: u32, Env>(
    array: [T; N],
    ordering: fn[Env](T, T) -> bool,
) -> [SortedTuple<T>; N] {
    let mut tuples = [SortedTuple { elem: array[0], original_index: 0 }; N];
    for i in 0..N {
        tuples[i] = SortedTuple { elem: array[i], original_index: i };
    }
    sort_by(
        tuples,
        |a: SortedTuple<T>, b: SortedTuple<T>| ordering(a.elem, b.elem),
    )
}

#[test]
unconstrained fn get_sorted_tuple_asc() {
    let original = [3, 2, 9, 5];
    let expected = [
        SortedTuple { elem: 2, original_index: 1 },
        SortedTuple { elem: 3, original_index: 0 },
        SortedTuple { elem: 5, original_index: 3 },
        SortedTuple { elem: 9, original_index: 2 },
    ];
    let sorted = get_sorted_tuple(original, |a: u64, b: u64| a < b);
    for i in 0..4 {
        assert_eq(sorted[i].elem, expected[i].elem);
        assert_eq(sorted[i].original_index, expected[i].original_index);
    }
}
