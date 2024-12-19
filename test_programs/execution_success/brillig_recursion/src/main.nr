// Tests a very simple program.
//
// The feature being tested is brillig recursion
fn main(x: u32) {
    unsafe {
        //@safety: testing context
        assert(fibonacci(x) == 55);
    }
}

unconstrained fn fibonacci(x: u32) -> u32 {
    if x <= 1 {
        x
    } else {
        fibonacci(x - 1) + fibonacci(x - 2)
    }
}
