struct Foo<T> {
    x: T,
}

type Bar<U> = Foo<U>;

fn main() {
    let Bar { x } = Foo { x: [0] };
    // This is just to show the compiler knows this is an array.
    let _: [Field; 1] = x;
}
