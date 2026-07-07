struct Params<T> {}

trait Trait {
    type T;
    fn params() -> Params<T>;
}

pub struct Foo {}

impl Trait for Foo {
    type T = i32;

    fn params() -> Params<i32> {
        Params {}
    }
}

fn main() {
    let params = <Foo as Trait>::params();
    foo(params); // Could not determine the type of the generic argument `T` declared on the function `foo`
}

fn foo<T>(_: Params<T>) {}
