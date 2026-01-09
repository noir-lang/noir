// TODO: WIP

enum Foo<T> {
    Bar(T),
}

impl Foo<1> {}

fn main() {
    let foo: Foo<1> = std::mem::zeroed();
    println(foo);
}
