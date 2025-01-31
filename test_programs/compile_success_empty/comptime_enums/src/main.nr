fn main() {
    comptime {
        let _two = Foo::Couple(1, 2);
        let _one = Foo::One(3);
        let _none = Foo::None;
    }
}

enum Foo {
    Couple(i32, i32),
    One(i32),
    None,
}
