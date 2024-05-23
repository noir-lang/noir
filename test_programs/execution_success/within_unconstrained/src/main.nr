use dep::std::within_unconstrained;

fn check(should_be_unconstrained: bool) {
    assert_eq(should_be_unconstrained, within_unconstrained());
}

unconstrained fn unconstrained_intermediate() {
    check(true);
}

fn main() {
    unconstrained_intermediate();
    check(false);
}
