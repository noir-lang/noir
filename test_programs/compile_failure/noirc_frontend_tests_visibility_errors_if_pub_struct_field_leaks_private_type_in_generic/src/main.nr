
    pub mod moo {
        struct Bar {}
        pub struct Foo<T> { pub value: T }
        pub struct FooBar { pub value: Foo<Bar> }

        pub fn no_unused_warnings() {
            let _ = FooBar { value: Foo { value: Bar {} } };
        }
    }
    fn main() {}
    