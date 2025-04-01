
    trait Foo<T> {
        fn foo<U>() where T: T2<T>;
    }

    impl<A> Foo<A> for () {
        // Should be A: T2<A>
        fn foo<B>() where A: T2<B> {}
    }

    trait T2<C> {}
    