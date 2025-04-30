
    struct foo {}

    fn foo(x: foo) -> foo {
        x
    }

    fn main() {
        let x: foo = foo {};
        let _ = foo(x);
    }
    