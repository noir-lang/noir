pub trait Foo: Eq {
    fn foo<T>(self) -> bool {
        self == self
    }
}

fn main() {}
