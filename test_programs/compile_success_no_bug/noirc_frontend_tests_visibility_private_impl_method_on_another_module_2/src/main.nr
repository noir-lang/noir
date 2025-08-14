
    pub mod bar {
        pub struct Foo<T> {}
    }

    impl bar::Foo<i32> {
        fn foo(self) {
            let _ = self;
        }
    }

    impl bar::Foo<i64> {
        fn bar(self) {
            let _ = self;
            let foo = bar::Foo::<i32> {};
            foo.foo();
        }
    }

    fn main() {}
    