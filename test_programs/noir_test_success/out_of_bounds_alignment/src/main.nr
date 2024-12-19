fn out_of_bounds(arr_1: [Field; 50]) -> Field {
    arr_1[50 + 1]
}

unconstrained fn out_of_bounds_unconstrained_wrapper(arr_1: [Field; 50], arr_2: [Field; 50]) -> Field {
    out_of_bounds(arr_1)
}

#[test(should_fail)]
fn test_acir() {
    assert_eq(out_of_bounds([0; 50]), 0);
}

#[test(should_fail)]
fn test_brillig() {
    unsafe {
        //@safety: testing context
        assert_eq(out_of_bounds_unconstrained_wrapper([0; 50], [0; 50]), 0);
    }
}
