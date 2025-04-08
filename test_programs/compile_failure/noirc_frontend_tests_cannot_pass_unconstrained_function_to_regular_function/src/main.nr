
    fn main() {
        let func = foo;
        expect_regular(func);
    }

    unconstrained fn foo() {}

    fn expect_regular(_func: fn() -> ()) {
    }
    