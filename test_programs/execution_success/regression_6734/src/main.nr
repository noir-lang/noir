// https://github.com/noir-lang/noir/issues/6734

pub fn sub_array_extended<let SRC_LEN: u32, let DST_LEN: u32>(
    src: [Field; SRC_LEN],
    offset: u32,
) -> [Field; DST_LEN] {
    let available_elements_to_copy = SRC_LEN - offset;
    let elements_to_copy = if DST_LEN > available_elements_to_copy {
        available_elements_to_copy
    } else {
        DST_LEN
    };

    let mut dst: [Field; DST_LEN] = std::mem::zeroed();
    for i in 0..elements_to_copy {
        dst[i] = src[i + offset];
    }

    dst
}

unconstrained fn main() {
    assert_eq(sub_array_extended([], 0), []);
}
