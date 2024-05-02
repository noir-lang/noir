// In `nargo test` we want to avoid the need for an external oracle resolver service to be required in the situation
// where its existence doesn't affect whether the tests will pass or fail. We then want to be able to handle any
// oracles which return zero field elements.

// Note that this custom oracle doesn't return any new values into the program.
// We can then safely continue execution even in the case where there is no oracle resolver to handle it.
#[oracle(custom_debug)]
unconstrained fn custom_debug() {}

// However this oracle call should return a field element. We expect the ACVM to raise an error when it
// doesn't receive this value.
#[oracle(custom_getter)]
unconstrained fn custom_getter() -> Field {}

#[test]
unconstrained fn unit_return_oracle_ignored() {
    custom_debug();
}

#[test(should_fail_with = "0 output values were provided as a foreign call result for 1 destination slots")]
unconstrained fn field_return_oracle_fails() {
    let _ = custom_getter();
}
