
    trait Foo<T> {
        fn foo(_x: T) -> Self;
    }

    impl<T> Foo<T> for i32 {
        fn foo(_x: T) -> Self {
            1
        }
    }

    fn main() {
        let _: i32 = Foo::<bool>::foo(1);
    }
    