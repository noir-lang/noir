
    struct U60Repr<let N: u32, let NumSegments: u32> {}

    impl<let N: u32, let NumSegments: u32> U60Repr<N, NumSegments> {
        fn new<let NumFieldSegments: u32>(_: [Field; N * NumFieldSegments]) -> Self {
            U60Repr {}
        }
    }

    fn main() {
        let input: [Field; 6] = [0; 6];
        let _: U60Repr<3, 6> = U60Repr::new(input);
    }
    