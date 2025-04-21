
    trait MyTrait {
        fn good_foo<T, H>() where H: OtherTrait;

        fn bad_foo<T, H>() where H: OtherTrait;
    }

    trait OtherTrait {}

    struct Option2<T> {
        inner: T
    }

    impl<T> MyTrait for [T] where Option2<T>: MyTrait {
        fn good_foo<A, B>() where B: OtherTrait { }

        fn bad_foo<A, B>() where A: OtherTrait { }
    }

    fn main() {
        let _ = Option2 { inner: 1 }; // silence Option2 never constructed warning
    }
    