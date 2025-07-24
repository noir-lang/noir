
    pub trait From2<T> {}

    impl<T> From2<T> for T {}

    pub trait Into2<T> {}

    impl From2<u8> for Field {}

    impl<T: From2<U>, U> Into2<T> for U {}

    fn foo<T: Into2<Field>>() {}

    fn main() {
        foo::<u8>();
    }
    