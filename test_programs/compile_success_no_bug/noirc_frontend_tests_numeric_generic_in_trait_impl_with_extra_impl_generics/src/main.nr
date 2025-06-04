
    trait Default2 {
        fn default2() -> Self;
    }

    struct MyType<T> {
        a: Field,
        b: Field,
        c: Field,
        d: T,
    }

    // Make sure that `T` is placed before `N` as we want to test that the order of the generics is correctly maintained.
    // `N` is used first in the trait impl generics (`Deserialize<N> for MyType<T>`).
    // We want to make sure that the compiler correctly accounts for that `N` has a numeric kind
    // while `T` has a normal kind.
    impl<T, let N: u32> Deserialize<N> for MyType<T> where T: Default2 {
        fn deserialize(fields: [Field; N]) -> Self {
            MyType { a: fields[0], b: fields[1], c: fields[2], d: T::default2() }
        }
    }

    trait Deserialize<let N: u32> {
        fn deserialize(fields: [Field; N]) -> Self;
    }

    fn main() { }
    