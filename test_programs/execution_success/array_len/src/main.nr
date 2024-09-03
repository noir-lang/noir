fn len_plus_1<T, let N: u32>(array: [T; N]) -> u32 {
    array.len() + 1
}

fn add_lens<T, let N: u32, let M: u32>(a: [T; N], b: [Field; M]) -> u32 {
    a.len() + b.len()
}

fn nested_call<let N: u32>(b: [Field; N]) -> u32 {
    len_plus_1(b)
}

fn main(x: Field, len3: [u8; 3], len4: [Field; 4]) {
    assert(len_plus_1(len3) == 4);
    assert(len_plus_1(len4) == 5);
    assert(add_lens(len3, len4) == 7);
    assert(nested_call(len4) == 5);
    // std::array::len returns a compile-time known value
    assert(len4[len3.len()] == 4);
    // Regression for #1023, ensure .len still works after calling to_le_bytes on a witness.
    // This was needed because normally .len is evaluated before acir-gen where to_le_bytes
    // on a witness is only evaluated during/after acir-gen.
    assert(x.to_le_bytes::<8>().len() != 0);
}
