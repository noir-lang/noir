
        pub trait Trait<T> {}
        pub struct Foo<T> {}

        impl<T, U> Trait<T> for Foo<T> {}

        fn main() {}
        