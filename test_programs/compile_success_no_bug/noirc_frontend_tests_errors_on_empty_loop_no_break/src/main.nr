
    fn main() {
        // Safety: test
        unsafe {
            foo()
        }
    }

    unconstrained fn foo() {
        loop {}
    }
    