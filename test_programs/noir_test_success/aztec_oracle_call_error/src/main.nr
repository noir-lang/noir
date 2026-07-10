#[oracle(aztec_oracle_call)]
unconstrained fn oracle_call() {}


#[test(should_fail_with = "Oracle call `aztec_oracle_call` requires connection connection to an Aztec simulation environment. See https://foo.bar/aztec for more information.")]
unconstrained fn test_without_pxe() {
    oracle_call();
}
