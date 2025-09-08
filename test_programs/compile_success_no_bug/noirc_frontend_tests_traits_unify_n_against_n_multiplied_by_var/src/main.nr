
    trait Serialize {
        let N: u32;

        fn serialize(self) -> [u32; N];
    }

    impl Serialize for Field {
        let N: u32 = 1;

        fn serialize(self) -> [u32; Self::N] {
            [0; 1]
        }
    }

    impl<let N: u32, T: Serialize> Serialize for [T; N] {
        let N: u32 = <T as Serialize>::N * N;

        fn serialize(self) -> [u32; Self::N] {
            [0; Self::N]
        }
    }

    pub struct CompressedString<let N: u32> {
        value: [Field; N],
    }

    impl<let N: u32> Serialize for CompressedString<N> {
        let N: u32 = N;

        fn serialize(self) -> [u32; Self::N] {
            // In this line we'll end up trying to eventually unify
            // `N` against `N * var`, which should bind `var` to `1`.
            let _: [u32; N] = self.value.serialize();
            [0; Self::N]
        }
    }

    fn main() {}
    