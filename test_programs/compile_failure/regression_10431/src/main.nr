struct Foo<T> {
    x: T,
}

impl Foo<1> {}

fn main() {
    let foo: Foo<1> = std::mem::zeroed();
    println(foo.x);
}
