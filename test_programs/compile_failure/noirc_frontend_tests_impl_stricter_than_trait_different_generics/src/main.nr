
    trait Default2 { }

    // Object type of the trait constraint differs
    trait Foo<T> {
        fn foo_good<U>() where T: Default2;

        fn foo_bad<U>() where T: Default2;
    }

    impl<A> Foo<A> for () {
        fn foo_good<B>() where A: Default2 {}

        fn foo_bad<B>() where B: Default2 {}
    }
    