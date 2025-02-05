global ONE: Field = 1;
global TWO: Field = 2;
global THREE: Field = 3;

fn main(x: Field, y: pub Field) {
    /// Safety: testing context
    unsafe {
        entry_point_no_global(x, y);
        entry_point_inner_func_globals(x, y);
        entry_point_one_global(x, y);
        entry_point_one_diff_global(x, y);
    }
}

unconstrained fn entry_point_no_global(x: Field, y: Field) {
    assert(x + y != 100);
}

unconstrained fn entry_point_one_global(x: Field, y: Field) {
    let z = TWO + x + y;
    assert(z == 3);
}

unconstrained fn entry_point_inner_func_globals(x: Field, y: Field) {
    wrapper(x, y);
}

// Test that we duplicate Brillig entry points called within
// another entry point's inner calls
unconstrained fn wrapper(x: Field, y: Field) {
    let z = ONE + x + y;
    assert(z == 2);
    entry_point_one_global(x, y);
    // Test that we handle repeated entry point calls
    // `entry_point_one_diff_global` should be duplicated and the duplicated function
    // should use the globals from `entry_point_inner_func_globals`
    entry_point_one_diff_global(y, x);
}

unconstrained fn entry_point_one_diff_global(x: Field, y: Field) {
    let z = THREE + x + y;
    assert(z == 4);
}

