use dep::std::slice;

// Tests oracle usage in brillig/unconstrained functions
fn main(x: Field)  {
    // call through a brillig wrapper
    oracle_print_array_wrapper([x, x]);

    // TODO(#1615) Nargo currently only supports resolving one foreign call
    // oracle_print_wrapper(x);

    get_number_sequence_wrapper(20);
}

// TODO(#1911)
#[oracle(oracle_print_impl)]
unconstrained fn oracle_print(_x : Field) -> Field {}

unconstrained fn oracle_print_wrapper(x: Field) {
    oracle_print(x);
}

// TODO(#1911)
#[oracle(oracle_print_array_impl)]
unconstrained fn oracle_print_array(_arr : [Field; 2]) -> Field {}

unconstrained fn oracle_print_array_wrapper(arr: [Field; 2]) {
    oracle_print_array(arr);
}

// TODO(#1911): This function does not need to be an oracle but acts 
// as a useful test while we finalize code generation for slices in Brillig
#[oracle(get_number_sequence)]
unconstrained fn get_number_sequence(_size: Field) -> [Field] {}

// TODO(#1911)
#[oracle(get_reverse_number_sequence)]
unconstrained fn get_reverse_number_sequence(_size: Field) -> [Field] {}

unconstrained fn get_number_sequence_wrapper(size: Field) {
    let slice = get_number_sequence(size);
    for i in 0..19 as u32 {
        assert(slice[i] == i as Field);
    }

    let reversed_slice = get_reverse_number_sequence(size);
    // Regression test that we have not overwritten memory
    for i in 0..19 as u32 {
        assert(slice[i] == reversed_slice[19 - i]);
    }
}

