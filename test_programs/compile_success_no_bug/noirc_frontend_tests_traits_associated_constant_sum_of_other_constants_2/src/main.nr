
    pub trait Deserialize {
        let N: u32;

        fn deserialize(_: [Field; N]);
    }

    impl Deserialize for Field {
        let N: u32 = 1;

        fn deserialize(_: [Field; Self::N]) {}
    }

    impl<T, let M: u32> Deserialize for [T; M]
    where
        T: Deserialize,
    {
        let N: u32 = <T as Deserialize>::N + M;

        fn deserialize(_: [Field; Self::N]) {}
    }

    pub fn foo<let X: u32>() {
        let f = <[Field; X] as Deserialize>::deserialize;
        let _ = f([0; X + 1]);
    }

    fn main() {}
    