
    trait Default { }

    trait OtherDefault { }

    struct Option<T> {
        inner: T
    }

    trait Bar<T> {
        fn bar<U>() where Option<T>: Default;
    }

    impl<A> Bar<A> for () {
        // Trait constraint differs due to the trait even though the constraint
        // types are the same.
        fn bar<B>() where Option<A>: OtherDefault {}
    }

    fn main() {
        let _ = Option { inner: 1 }; // silence Option never constructed warning
    }
    