
    struct Reader<let N: u32> {
    }

    impl<let N: u32> Reader<N> {
        fn read<let C: u32>(_self: Self) {}
    }

    fn main() {
        let reader: Reader<1234> = Reader {};
        let _ = reader.read::<1234>();
    }
    