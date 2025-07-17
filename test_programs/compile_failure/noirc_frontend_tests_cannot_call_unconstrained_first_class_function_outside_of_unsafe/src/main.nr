
    fn main() {
        let func = foo;
        func();
        inner(func);
    }

    fn inner(x: unconstrained fn() -> ()) {
        x();
    }

    unconstrained fn foo() {}
    