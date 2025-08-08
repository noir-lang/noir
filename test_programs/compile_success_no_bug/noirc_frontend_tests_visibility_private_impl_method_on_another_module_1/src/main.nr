
    pub mod bar {
        pub struct Foo<T> {}
    }

    impl<T> bar::Foo<T> {
        fn foo(self) {
            let _ = self;
        }

        fn bar(self) {
            self.foo();
        }
    }

    fn main() {}
    