// Tests oracle usage in brillig/unconstrained functions
fn main(x: Field)  {
    // call through a brillig wrapper
    // oracle_print_array_wrapper([x, x]);

    // TODO(#1615) Nargo currently only supports resolving one foreign call
    // oracle_print_wrapper(x);

    assert(x == oracle_identity_wrapper(x));

    // let arr = oracle_identity_array_wrapper([x, x]);
    // assert(x == arr[0]);
    // assert(x == arr[1]);
}

#[oracle(oracle_print_impl)]
unconstrained fn oracle_print(_x : Field) -> Field {}

unconstrained fn oracle_print_wrapper(x: Field) {
    oracle_print(x);
}

#[oracle(oracle_print_array_impl)]
unconstrained fn oracle_print_array(_arr : [Field; 2]) -> Field {}

unconstrained fn oracle_print_array_wrapper(arr: [Field; 2]) {
    oracle_print_array(arr);
}

#[oracle(oracle_identity)]
unconstrained fn oracle_identity(_x : Field) -> Field {}

unconstrained fn oracle_identity_wrapper(x: Field) -> Field {
    oracle_identity(x)
}

#[oracle(oracle_identity_array)]
unconstrained fn oracle_identity_array(_arr : [Field; 2]) -> [Field; 2] {}

unconstrained fn oracle_identity_array_wrapper(arr: [Field; 2]) -> [Field; 2] {
    oracle_identity_array(arr)
}
