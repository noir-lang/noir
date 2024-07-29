fn main() {
    let (first, rest) = split_first([1, 2, 3, 4]);
    assert_eq(first, 1);
    assert_eq(rest, [2, 3, 4]);

    // Type inference works without the type constraints from assert_eq as well
    let _ = split_first([1, 2, 3]);
}

fn split_first<T, let N: u32>(array: [T; N]) -> (T, [T; N - 1]) {
    std::static_assert(N != 0, "split_first called on empty array");
    let mut new_array: [T; N - 1] = std::unsafe::zeroed();

    for i in 0 .. N - 1 {
        new_array[i] = array[i + 1];
    }

    (array[0], new_array)
}
