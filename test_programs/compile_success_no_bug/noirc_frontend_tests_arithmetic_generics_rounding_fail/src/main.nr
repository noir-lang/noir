
        fn main() {
            // Do not simplify N/M*M to just N
            // This should be 3/2*2 = 2, not 3
            round::<3, 2>([1, 2, 3]);
        }

        fn round<let N: u32, let M: u32>(_x: [Field; N / M * M]) {}
    