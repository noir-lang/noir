
    fn some_func() {}

    mod foo {
        pub fn func() {
            super::some_func();
        }
    }

    fn main() { }
    