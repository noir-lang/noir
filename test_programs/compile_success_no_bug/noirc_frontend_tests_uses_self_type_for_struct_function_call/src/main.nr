
    struct S { }

    impl S {
        fn one() -> Field {
            1
        }

        fn two() -> Field {
            Self::one() + Self::one()
        }
    }

    fn main() {
        let _ = S {}; // silence S never constructed warning
    }
    