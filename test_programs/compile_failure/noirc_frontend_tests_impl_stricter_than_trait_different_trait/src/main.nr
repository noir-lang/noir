
    trait Default2 { }

    trait OtherDefault { }

    struct Option2<T> {
        inner: T
    }

    trait Bar<T> {
        fn bar<U>() where Option2<T>: Default2;
    }

    impl<A> Bar<A> for () {
        // Trait constraint differs due to the trait even though the constraint
        // types are the same.
        fn bar<B>() where Option2<A>: OtherDefault {}
    }

    fn main() {
        let _ = Option2 { inner: 1 }; // silence Option2 never constructed warning
    }
    