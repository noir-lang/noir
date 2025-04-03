
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    pub fn read<T, N>() -> T where T: Deserialize<N> {
        T::deserialize([0, 1])
    }
    