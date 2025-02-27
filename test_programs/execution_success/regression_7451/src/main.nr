fn main(x: u8) {
    // Safety: test code
    let predicate = unsafe { return_true() };
    let tmp: u8 = if predicate { 0xff } else { 0 };

    let y = tmp & x;
    assert_eq(y, 1);
}

unconstrained fn return_true() -> bool {
    true
}
