
    pub struct Foo {}
    pub type Bar = Foo<i32>;
    fn main() {
        let _ = Foo {}; // silence Foo never constructed warning
    }
    