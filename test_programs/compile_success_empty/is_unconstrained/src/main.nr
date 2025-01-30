use std::runtime::is_unconstrained;

fn check(should_be_unconstrained: bool) {
    assert_eq(should_be_unconstrained, is_unconstrained());
}

unconstrained fn unconstrained_intermediate() {
    check(true);
}

fn main() {
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        unconstrained_intermediate();
    }
    check(false);
}
