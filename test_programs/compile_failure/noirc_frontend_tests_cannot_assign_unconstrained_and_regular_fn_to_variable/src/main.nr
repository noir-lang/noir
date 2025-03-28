
    fn main() {
        let _func = if true { foo } else { bar };
    }

    fn foo() {}
    unconstrained fn bar() {}
    