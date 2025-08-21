
    trait Bar {
        fn baz<let N: u32>();
    }

    impl Bar for Field {
        fn baz<let N: u32>() {
            let _ = N;
        }
    }

    fn foo<K: Bar>() {
        K::baz::<2>();
    }

    fn main() {
        foo::<Field>();
    }
    