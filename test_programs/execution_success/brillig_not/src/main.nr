// Tests a very simple Brillig function.
//
// The features being tested is not instruction on brillig
fn main(x: Field, y: Field) {
    // Safety: testing context
    unsafe {
        assert(false == not_operator(x != 0));
        assert(true == not_operator(y != 0));
    }
}

unconstrained fn not_operator(x: bool) -> bool {
    !x
}
