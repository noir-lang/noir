fn main(x: u64) {
    acir_wrapper(x);
}

fn acir_wrapper(x: u64) {
    unsafe {
        brillig_entrypoint(x);
    }
}

unconstrained fn brillig_entrypoint(x: u64) {
    brillig_nested(x);
}

unconstrained fn brillig_nested(x: u64) {
    assert(x < 10);
}
