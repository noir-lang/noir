
    struct Foo {}
    pub type Bar = Foo;
    pub fn no_unused_warnings() {
        let _: Bar = Foo {};
    }
    fn main() {}
    