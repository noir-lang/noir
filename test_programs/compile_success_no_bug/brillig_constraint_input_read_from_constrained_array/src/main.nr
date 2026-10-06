unconstrained fn copy_helper(x: [Field; 4]) -> [Field; 4] {
    x
}

unconstrained fn half_helper(x: Field) -> Field {
    x / 2
}

fn main(x: [Field; 4], i: u32) -> pub Field {
    // Safety: every element is constrained below
    let copy = unsafe { copy_helper(x) };
    for j in 0..4 {
        assert_eq(copy[j], x[j]);
    }
    let value = copy[i];
    // Safety: constrained below
    let half = unsafe { half_helper(value) };
    assert_eq(half * 2, value);
    half
}
