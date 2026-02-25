// Regression test: Brillig crash with mutable references inside arrays in a loop.
// "value is not typed as Brillig usize" when dereferencing after loop mutation.
fn main() -> pub [Field; 1] {
    unsafe {
        foo()
    }
}

unconstrained fn foo() -> [Field; 1] {
    bar([&mut [0], &mut [0]])
}

unconstrained fn bar(mut a: [&mut [Field; 1]; 2]) -> [Field; 1] {
    {
        let mut running = true;
        while running {
            a = [&mut [0], a[0]];
            running = false;
        }
    };
    *a[0]
}
