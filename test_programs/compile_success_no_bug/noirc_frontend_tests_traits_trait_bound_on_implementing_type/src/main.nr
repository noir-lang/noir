
    struct GenericStruct<T> {
        inner: T,
    }

    trait Foo {
        fn foo() {}
    }

    impl Foo for Field {}

    impl<T: Foo> Foo for GenericStruct<T> {}

    trait Bar {
        fn bar();
    }

    impl<T> Bar for GenericStruct<T>
    where
        GenericStruct<T>: Foo,
    {
        fn bar() {
            <Self as Foo>::foo()
        }
    }
    
    fn main() {
        GenericStruct::<Field>::bar();
    }
    