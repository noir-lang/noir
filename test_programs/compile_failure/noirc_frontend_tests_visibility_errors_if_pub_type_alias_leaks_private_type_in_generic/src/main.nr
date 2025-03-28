
    pub mod moo {
        struct Bar {}
        pub struct Foo<T> { pub value: T }
        pub type FooBar = Foo<Bar>;

        pub fn no_unused_warnings() {
            let _: FooBar = Foo { value: Bar {} };
        }
    }
    fn main() {}
    