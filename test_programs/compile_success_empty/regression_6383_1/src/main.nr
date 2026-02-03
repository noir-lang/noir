trait Foo {
    type Associated;

    fn foo(self) -> u8;
}

pub struct Bar<T> {
    inner: T,
}

impl<T> Foo for Bar<T>
where
    T: Foo,
{
    type Associated = ();

    fn foo(self) -> u8 {
        1 + self.inner.foo()
    }
}

fn main() {}
