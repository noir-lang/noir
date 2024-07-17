struct SortedTuple<T> {
    value: T,
    original_index: u32,
}

pub fn sort_high_to_low<let N: u32, T>(values: [T; N], is_less_than: fn(T, T) -> bool) -> [SortedTuple<T>; N] where T: Eq {
    let mut sorted_tuples = [SortedTuple { value: values[0], original_index: 0 }; N];

    for i in 0..N {
        sorted_tuples[i] = SortedTuple {
            value: values[i],
            original_index: i,
        };
    }

    sorted_tuples.sort_via(|a: SortedTuple<T>, b: SortedTuple<T>| (b.value == a.value) | is_less_than(b.value, a.value))
}
