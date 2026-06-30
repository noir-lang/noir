//! Regression test: a user-defined oracle whose name equals the name the proxy pass
//! generates for another oracle used as a value.
//!
//! When `oracle_hash` is passed as a function value, `create_foreign_proxies` generates an
//! internal function named `oracle_hash_proxy` that forwards to the `oracle_hash` oracle. That
//! name collides with the user-defined `oracle_hash_proxy` oracle. The collision is only in the
//! printed name: foreign calls dispatch by oracle name string and proxies dispatch by `FuncId`,
//! so each call must still reach its intended oracle.
use std::test::OracleMock;

#[oracle(oracle_hash)]
unconstrained fn oracle_hash(_input: Field) -> Field {}

#[oracle(oracle_hash_proxy)]
unconstrained fn oracle_hash_proxy(_input: Field) -> Field {}

unconstrained fn call_with(f: unconstrained fn(Field) -> Field, input: Field) -> Field {
    f(input)
}

#[test]
unconstrained fn dispatches_to_correct_oracle_despite_proxy_name_collision() {
    let _ = OracleMock::mock("oracle_hash").returns(1);
    let _ = OracleMock::mock("oracle_hash_proxy").returns(2);

    // Passing `oracle_hash` as a value generates an internal `oracle_hash_proxy` function that
    // forwards to the `oracle_hash` oracle, so this must observe `oracle_hash`'s mock (1).
    assert_eq(call_with(oracle_hash, 0), 1);

    // Passing the user-defined `oracle_hash_proxy` oracle as a value must still reach its own
    // oracle (2), not the internal proxy that forwards to `oracle_hash`.
    assert_eq(call_with(oracle_hash_proxy, 0), 2);

    // A direct call to `oracle_hash_proxy` must reach its oracle (2) as well.
    assert_eq(oracle_hash_proxy(0), 2);
}
