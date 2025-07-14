
    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    pub fn read<T, let N: u32>() -> T where T: Deserialize<N> {
        let mut fields: [Field; N] = [0; N];
        for i in 0..N {
            fields[i] = i as Field + 1;
        }
        T::deserialize(fields)
    }

    fn main() { }
    