
        struct S<let N: u8> {}

        fn main() {
            let _: S<3> = S {};
        }
    