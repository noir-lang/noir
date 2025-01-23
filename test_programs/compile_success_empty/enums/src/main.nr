fn main() {
    let _a = Foo::A::<u8>(1, 2);
    let _b: Foo<u16> = Foo::B(3);
    let _c = Foo::C(4);
    let _d = Foo::D::<(i32, i32)>;

    // Can't do anything with them yet...
}

enum Foo<T> {
    A(Field, Field),
    B(u32),
    C(T),
    D,
}
