struct Foo<T> {}

impl Foo<u32> {
    fn foo(_self: Self) -> field {
        1
    }
}

impl Foo<u64> {
    fn foo(_self: Self) -> field {
        2
    }
}

fn main() {
    let f1: Foo<u32> = Foo {};
    let f2: Foo<u64> = Foo {};
    assert(f1.foo() + f2.foo() == 3);
}
