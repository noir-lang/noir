// `mapi` over a runtime-length vector whose closure returns a multi-field struct.
struct Entry {
    amount: u64,
    index: u32,
    value: Field,
    tag: u8,
    first: bool,
}

fn main(xs: [Field; 32], len: u32) -> pub Field {
    let vec: BoundedVec<Field, 32> = BoundedVec::from_parts(xs, len);
    let mapped = vec.mapi(|i, x| {
        Entry { amount: x as u64, index: i, value: x * 2, tag: i as u8, first: i == 0 }
    });

    assert_eq(mapped.len(), len);
    let mut acc = 0;
    for i in 0..32 {
        let e = mapped.get_unchecked(i);
        if i < len {
            assert_eq(e.value, xs[i] * 2);
            assert_eq(e.index, i);
        } else {
            assert_eq(e.value, 0);
            assert_eq(e.amount, 0);
        }
        acc += e.amount as Field + e.index as Field + e.value + e.tag as Field + e.first as Field;
    }
    acc
}
