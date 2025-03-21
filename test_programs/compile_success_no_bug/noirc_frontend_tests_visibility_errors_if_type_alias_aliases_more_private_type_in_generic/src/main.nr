
    pub struct Generic<T> { value: T }
    struct Foo {}
    pub type Bar = Generic<Foo>;
    pub fn no_unused_warnings() {
        let _ = Foo {};
        let _: Bar = Generic { value: Foo {} };
    }
    fn main() {}
    