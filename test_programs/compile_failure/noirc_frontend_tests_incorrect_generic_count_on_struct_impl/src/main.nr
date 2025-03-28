
    struct Foo {}
    impl <T> Foo<T> {}
    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    