
    mod foo {
        pub trait Foo {}
    }

    use foo::Foo;

    pub trait Bar<T>
    where
        T: Foo,
    {}

    fn main() {}
    