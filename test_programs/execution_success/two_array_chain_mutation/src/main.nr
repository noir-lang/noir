// Regression for https://github.com/noir-lang/noir/issues/12411.
// The pattern (two arrays filled in the same loop and returned together) previously
// produced O(N^2) ACIR memory opcodes because `mutable_array_set_optimization`'s
// is_nested heuristic refused to mark any array_set as mutable. A soundness bug in
// the corrected optimization would cause the two chains to corrupt each other, which
// the assertions below would catch.
fn fill_local<let N: u32>(enable: [bool; N]) -> ([Field; N], [Field; N]) {
    let mut data_a: [Field; N] = [0; N];
    let mut data_b: [Field; N] = [0; N];
    let mut ptr: u32 = 0;
    for i in 0..N {
        data_a[ptr] = i as Field;
        data_b[ptr] = i as Field;
        ptr += enable[i] as u32;
    }
    (data_a, data_b)
}

fn main(enable: [bool; 32]) {
    let (a, b) = fill_local(enable);
    for i in 0..32 {
        assert(a[i] == b[i]);
    }
    assert(a[0] == 0);
    assert(a[15] == 15);
    assert(a[31] == 31);
}
