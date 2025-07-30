trait Foo {
    type Associated;

    fn foo() -> u8;
}

pub struct Bar<T> {
    inner: T,
}

impl<T> Foo for Bar<T>
where
    T: Foo,
{
    type Associated = ();

    fn foo() -> u8 {
        1 + T::foo()
    }
}

fn main() {}
