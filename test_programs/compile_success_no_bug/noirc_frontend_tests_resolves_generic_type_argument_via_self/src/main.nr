
    pub struct Foo<T> {}

    impl<T> Foo<T> {
        fn one() {
            Self::two();
        }

        fn two() {}
    }

    fn main() {
        Foo::<i32>::one();
    }
    