
    trait MyTrait { }

    trait OtherTrait {}

    struct Option2<T> {
        inner: T
    }

    struct OtherOption<T> {
        inner: Option2<T>,
    }

    trait Bar<T> {
        fn bar_good<U>() where Option2<T>: MyTrait, OtherOption<Option2<T>>: OtherTrait;

        fn bar_bad<U>() where Option2<T>: MyTrait, OtherOption<Option2<T>>: OtherTrait;

        fn array_good<U>() where [T; 8]: MyTrait;

        fn array_bad<U>() where [T; 8]: MyTrait;

        fn tuple_good<U>() where (Option2<T>, Option2<U>): MyTrait;

        fn tuple_bad<U>() where (Option2<T>, Option2<U>): MyTrait;
    }

    impl<A> Bar<A> for () {
        fn bar_good<B>()
        where
            OtherOption<Option2<A>>: OtherTrait,
            Option2<A>: MyTrait { }

        fn bar_bad<B>()
        where
            OtherOption<Option2<A>>: OtherTrait,
            Option2<B>: MyTrait { }

        fn array_good<B>() where [A; 8]: MyTrait { }

        fn array_bad<B>() where [B; 8]: MyTrait { }

        fn tuple_good<B>() where (Option2<A>, Option2<B>): MyTrait { }

        fn tuple_bad<B>() where (Option2<B>, Option2<A>): MyTrait { }
    }

    fn main() {
        let _ = OtherOption { inner: Option2 { inner: 1 } }; // silence unused warnings
    }
    