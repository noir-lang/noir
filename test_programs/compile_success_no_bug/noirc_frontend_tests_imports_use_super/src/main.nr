
    fn some_func() {}

    mod foo {
        use super::some_func;

        pub fn bar() {
            some_func();
        }
    }

    fn main() { }
    