// Tests a very simple program.
//
// The features being tested is brillig calls passing arrays around
fn main(x: [u32; 3]) {
    /// Safety: testing context
    unsafe {
        assert(entry_point(x) == 9);
        another_entry_point(x);
    }
}

unconstrained fn inner(x: [u32; 3]) -> [u32; 3] {
    [x[0] + 1, x[1] + 1, x[2] + 1]
}

unconstrained fn entry_point(x: [u32; 3]) -> u32 {
    let y = inner(x);
    y[0] + y[1] + y[2]
}

unconstrained fn nested_fn_that_allocates(value: u32) -> u32 {
    let x = [value, value, value];
    let y = inner(x);
    y[0] + y[1] + y[2]
}

unconstrained fn another_entry_point(x: [u32; 3]) {
    assert(x[0] == 1);
    assert(x[1] == 2);
    assert(x[2] == 3);
    assert(nested_fn_that_allocates(1) == 6);
    // x should be unchanged
    assert(x[0] == 1);
    assert(x[1] == 2);
    assert(x[2] == 3);
}
