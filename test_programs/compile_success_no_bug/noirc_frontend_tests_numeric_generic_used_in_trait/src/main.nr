
    struct MyType<T> {
        a: Field,
        b: Field,
        c: Field,
        d: T,
    }

    impl<let N: u32, T> Deserialize<N, T> for MyType<T> {
        fn deserialize(fields: [Field; N], other: T) -> Self {
            MyType { a: fields[0], b: fields[1], c: fields[2], d: other }
        }
    }

    trait Deserialize<let N: u32, T> {
        fn deserialize(fields: [Field; N], other: T) -> Self;
    }

    fn main() { }
    