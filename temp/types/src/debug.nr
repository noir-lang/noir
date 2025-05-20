// Temporary workaround to bypass compiler issues. Passing values to the oracle is able to fix some bugs such as https://github.com/noir-lang/noir/issues/7192
// and https://github.com/noir-lang/noir/issues/7192

#[oracle(noOp)]
unconstrained fn no_op_oracle<T>(value: T) {}

unconstrained fn no_op_oracle_wrapper<T>(value: T) {
    no_op_oracle(value);
}

pub fn no_op<T>(value: T) {
    // Safety: This is a no op
    unsafe {
        no_op_oracle_wrapper(value);
    }
}
