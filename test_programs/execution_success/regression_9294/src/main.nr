enum Foo {
    One(Field),
    Two(u32, u64),
    Three,
}

fn main() {
    let enums: [Foo; 3] = [Foo::One(10), Foo::Two(20, 30), Foo::Three];
    println((enums, 40));
    println(f"Hello {enums}!")
}
