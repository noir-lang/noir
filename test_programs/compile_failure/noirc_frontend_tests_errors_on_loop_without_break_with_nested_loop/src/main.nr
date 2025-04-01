
    fn main() {
        // Safety: test
        unsafe {
            foo()
        }
    }

    unconstrained fn foo() {
        let mut x = 1;
        loop {
            x += 1;
            bar(x);
            loop {
                x += 2;
                break;
            }
        }
    }

    fn bar(_: Field) {}
    