
    pub trait Trait {}

    trait Serialize {
        let Size: u32;

        fn serialize(self);
    }

    impl<A, B> Serialize for (A, B)
    where
        A: Serialize,
        B: Serialize,
    {
        let Size: u32 = <A as Serialize>::Size + <B as Serialize>::Size;

        fn serialize(self: Self) {
            self.0.serialize();
        }
    }

    impl<T, let N: u32> Serialize for [T; N]
    where
        T: Serialize,
    {
        let Size: u32 = <T as Serialize>::Size;

        fn serialize(self: Self) {
        }
    }

    impl Serialize for Field {
        let Size: u32 = 1;

        fn serialize(self) {
        }
    }

    fn main() {
        let x = (((1, [2, 3, 4]), [5, 6, 7, 8]), 9);
        x.serialize();
    }
    