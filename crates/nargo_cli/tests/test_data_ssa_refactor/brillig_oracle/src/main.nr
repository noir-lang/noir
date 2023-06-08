// Tests oracle usage in brillig/unconstrained functions
fn main(x: Field)  {
    // call through a brillig wrapper
    oracle_print_wrapper(x);
}


#[oracle(oracle_print_impl)]
unconstrained fn oracle_print(_x : Field) {}

unconstrained fn oracle_print_wrapper(x: Field) {
    oracle_print(x);
}


