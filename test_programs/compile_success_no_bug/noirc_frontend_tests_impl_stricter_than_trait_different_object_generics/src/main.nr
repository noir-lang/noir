
    trait MyTrait { }

    trait OtherTrait {}

    struct Option<T> {
        inner: T
    }

    struct OtherOption<T> {
        inner: Option<T>,
    }

    trait Bar<T> {
        fn bar_good<U>() where Option<T>: MyTrait, OtherOption<Option<T>>: OtherTrait;

        fn bar_bad<U>() where Option<T>: MyTrait, OtherOption<Option<T>>: OtherTrait;

        fn array_good<U>() where [T; 8]: MyTrait;

        fn array_bad<U>() where [T; 8]: MyTrait;

        fn tuple_good<U>() where (Option<T>, Option<U>): MyTrait;

        fn tuple_bad<U>() where (Option<T>, Option<U>): MyTrait;
    }

    impl<A> Bar<A> for () {
        fn bar_good<B>()
        where
            OtherOption<Option<A>>: OtherTrait,
            Option<A>: MyTrait { }

        fn bar_bad<B>()
        where
            OtherOption<Option<A>>: OtherTrait,
            Option<B>: MyTrait { }

        fn array_good<B>() where [A; 8]: MyTrait { }

        fn array_bad<B>() where [B; 8]: MyTrait { }

        fn tuple_good<B>() where (Option<A>, Option<B>): MyTrait { }

        fn tuple_bad<B>() where (Option<B>, Option<A>): MyTrait { }
    }

    fn main() {
        let _ = OtherOption { inner: Option { inner: 1 } }; // silence unused warnings
    }
    