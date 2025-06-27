struct Bar<T> {
    one: Field,
    two: Field,
    other: T,
}

impl<T> Bar<T> {
    fn zeroed<A>(self) -> A {
        std::mem::zeroed()
    }
}

fn foo<T>(bar: Bar<T>) {
    assert(bar.one == bar.two);
}

fn main(x: Field, y: pub Field) {
    let bar1: Bar<Field> = Bar { one: x, two: y, other: 0 };

    assert(bar1.zeroed::<u32, Field>() == 0);
}
