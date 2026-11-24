// `map` over a runtime-length vector whose closure returns a multi-field struct.
struct Entry {
    amount: u64,
    small: u32,
    value: Field,
    tag: u8,
    is_three: bool,
}

fn main(xs: [Field; 32], len: u32) -> pub Field {
    let vec: BoundedVec<Field, 32> = BoundedVec::from_parts(xs, len);
    let mapped = vec.map(|x| {
        Entry { amount: x as u64, small: x as u32, value: x * 2, tag: x as u8, is_three: x == 3 }
    });

    assert_eq(mapped.len(), len);
    let mut acc = 0;
    for i in 0..32 {
        let e = mapped.get_unchecked(i);
        if i < len {
            assert_eq(e.value, xs[i] * 2);
        } else {
            assert_eq(e.value, 0);
            assert_eq(e.amount, 0);
        }
        acc +=
            e.amount as Field + e.small as Field + e.value + e.tag as Field + e.is_three as Field;
    }
    acc
}
