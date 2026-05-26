// Regression: an oracle must not accept a reference parameter. Previously this passed the
// frontend and ICE'd in Brillig codegen ("unsupported return type for black box call").
unconstrained fn main(mut f: Field) {
    let result = call_oracle(&mut f);
    assert(result == 0);
}

#[oracle(get_ref)]
unconstrained fn oracle_get_ref(f: &mut Field) -> Field {}

unconstrained fn call_oracle(f: &mut Field) -> Field {
    oracle_get_ref(f)
}
