
    trait MyTrait {
        fn good_foo<T, H>() where H: OtherTrait;

        fn bad_foo<T, H>() where H: OtherTrait;
    }

    trait OtherTrait {}

    struct Option<T> {
        inner: T
    }

    impl<T> MyTrait for [T] where Option<T>: MyTrait {
        fn good_foo<A, B>() where B: OtherTrait { }

        fn bad_foo<A, B>() where A: OtherTrait { }
    }

    fn main() {
        let _ = Option { inner: 1 }; // silence Option never constructed warning
    }
    