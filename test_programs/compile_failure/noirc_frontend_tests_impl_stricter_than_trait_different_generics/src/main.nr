
    trait Default { }

    // Object type of the trait constraint differs
    trait Foo<T> {
        fn foo_good<U>() where T: Default;

        fn foo_bad<U>() where T: Default;
    }

    impl<A> Foo<A> for () {
        fn foo_good<B>() where A: Default {}

        fn foo_bad<B>() where B: Default {}
    }
    