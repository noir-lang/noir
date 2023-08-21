fn len_plus_1<T, N>(array: [T; N]) -> Field {
    array.len() + 1
}

fn add_lens<T, N, M>(a: [T; N], b: [Field; M]) -> Field {
    a.len() + b.len()
}

fn nested_call<N>(b: [Field; N]) -> Field {
    len_plus_1(b)
}

fn main(x: Field, len3: [u8; 3], len4: [Field; 4]) {
    assert_eq(len_plus_1(len3), 4);
    assert_eq(len_plus_1(len4), 5);
    assert_eq(add_lens(len3, len4), 7);
    assert_eq(nested_call(len4), 5);

    // std::array::len returns a compile-time known value
    assert_eq(len4[len3.len()], 4);

    // Regression for #1023, ensure .len still works after calling to_le_bytes on a witness.
    // This was needed because normally .len is evaluated before acir-gen where to_le_bytes
    // on a witness is only evaluated during/after acir-gen.
    assert(x.to_le_bytes(8).len() != 0);
}
