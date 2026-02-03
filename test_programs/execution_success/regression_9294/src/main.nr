enum Foo {
    Zero,
    One(Field),
    Two(u32, u64),
}

fn main() {
    let enums: [Foo; 3] = [Foo::Zero, Foo::One(10), Foo::Two(20, 30)];
    println((enums, 40));
    println(f"Hello {enums}!")
}
