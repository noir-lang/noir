#![inner]
#![inner2]

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

    #[custom]
    #[another_custom]
    mod tests {
        #[test]
        fn test() {
            data(1);
            data2(1);
        }
    }

    #[attr]
    mod baz;

    mod empty {
        #![inner]
    }
}

pub(crate) mod one { }

pub mod two { }
