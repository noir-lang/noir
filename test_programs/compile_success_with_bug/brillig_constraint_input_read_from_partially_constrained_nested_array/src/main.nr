unconstrained fn copy_helper(x: [[Field; 2]; 1]) -> [[Field; 2]; 1] {
    x
}

unconstrained fn half_helper(x: Field) -> Field {
    x / 2
}

fn main(x: [[Field; 2]; 1], i: u32) -> pub Field {
    // Safety: only the first value of the item is constrained
    let copy = unsafe { copy_helper(x) };
    let item = copy[0];
    assert_eq(item[0], x[0][0]);
    // `i` may select the unconstrained value
    let value = item[i];
    // Safety: constrained against `value`, which may be unconstrained
    let half = unsafe { half_helper(value) };
    assert_eq(half * 2, value);
    half
}
