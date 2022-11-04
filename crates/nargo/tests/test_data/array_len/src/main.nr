use dep::std;

fn len_plus_1<T>(array: [T]) -> Field {
    std::array::len(array) + 1
}

fn add_lens<T>(a: [T], b: [Field]) -> Field {
    std::array::len(a) + std::array::len(b)
}

fn nested_call(b: [Field]) -> Field {
    len_plus_1(b)
}

fn main(len3: [u8; 3], len4: [Field; 4]) {
    constrain len_plus_1(len3) == 4;
    constrain len_plus_1(len4) == 5;
    constrain add_lens(len3, len4) == 7;
    constrain nested_call(len4) == 5;

    // std::array::len returns a comptime value
    constrain len4[std::array::len(len3)] == 4;
}
