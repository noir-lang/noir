
    pub struct Foo<T> {}

    impl<T> Foo<T> {
        fn one() {
            Self::two();
        }

        fn two() {}
    }

    fn main() -> pub Field {
        let neg: Field = -1;
        Foo::<i32>::one();
        neg + 1
    }
    