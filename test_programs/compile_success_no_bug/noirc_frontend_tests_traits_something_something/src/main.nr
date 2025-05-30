
    trait Serialize {
        let N: u32;

        fn serialize(self) -> [Field; N];
    }

    impl Serialize for Field {
        let N: u32 = 1;

        fn serialize(self) -> [Field; Self::N] {
            [self]
        }
    }

    impl Serialize for i32 {
        let N = <Field as Serialize>::N;

        fn serialize(self) -> [Field; Self::N] {
            [0; Self::N]
        }
    }

    fn main() {}
    