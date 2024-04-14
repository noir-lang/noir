mod a {
    mod b {
        struct Data {
            a: Field
        }
    }

    fn data(a: Field) -> Data {
        Data { a }
    }

    fn data2(a: Field) -> Data2 {
        Data2 { a }
    }

    mod tests {
        #[test]
        fn test() {
            data(1);
            data2(1);
        }
    }
}
