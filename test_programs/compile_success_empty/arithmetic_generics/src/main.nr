fn main() {
    let (first, rest) = split_first([1, 2, 3, 4]);
    assert_eq(first, 1);
    assert_eq(rest, [2, 3, 4]);

    // Type inference works without the type constraints from assert_eq as well
    let _ = split_first([1, 2, 3]);

    let _ = push_multiple([1, 2, 3]);
}

fn split_first<T, let N: u32>(array: [T; N]) -> (T, [T; N - 1]) {
    std::static_assert(N != 0, "split_first called on empty array");
    let mut new_array: [T; N - 1] = std::unsafe::zeroed();

    for i in 0..N - 1 {
        new_array[i] = array[i + 1];
    }

    (array[0], new_array)
}

fn push<let N: u32>(array: [Field; N], element: Field) -> [Field; N + 1] {
    let mut result: [_; N + 1] = std::unsafe::zeroed();
    result[array.len()] = element;

    for i in 0..array.len() {
        result[i] = array[i];
    }

    result
}

fn push_multiple<let N: u32>(array: [Field; N]) -> [Field; N + 2] {
    // : [Field; N + 1]
    let array2 = push(array, 4);

    // : [Field; (N + 1) + 1]
    let array3 = push(array2, 5);

    // [Field; (N + 1) + 1] = [Field; N + 2]
    array3
}

// This signature fails because we can't match `_ + 1` to `3` at the call site
// fn push_multiple<let N: u32>(array: [Field; 1 + N]) -> [Field; N + 3] {
