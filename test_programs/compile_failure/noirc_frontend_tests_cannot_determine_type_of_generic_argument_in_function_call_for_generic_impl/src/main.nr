
    pub struct Foo<T> {}

    impl<T> Foo<T> {
        fn one() {}
    }
    
    fn main() {
        Foo::one();
    }
    