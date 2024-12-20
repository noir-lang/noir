fn main() {
    unsafe {
        //@safety: testing context
        assert_eq(fibonacci(3), fibonacci_hint(3));
    }
}

unconstrained fn fibonacci_hint(x: u32) -> u32 {
    fibonacci(x)
}

fn fibonacci(x: u32) -> u32 {
    if x <= 1 {
        x
    } else {
        fibonacci(x - 1) + fibonacci(x - 2)
    }
}
