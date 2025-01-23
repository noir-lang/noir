fn main() {
    let _a = Foo::A::<u8>(1, 2);
    let _b: Foo<u16> = Foo::B(3);
    let _c = Foo::C(4);

    // (#7172): Single variant enums must be called as functions currently
    let _d: fn() -> Foo<(i32, i32)> = Foo::D;
    let _d: Foo<(i32, i32)> = Foo::D();

    // Enum variants are functions and can be passed around as such
    let _many_cs = [1, 2, 3].map(Foo::C);
}

enum Foo<T> {
    A(Field, Field),
    B(u32),
    C(T),
    D,
}
