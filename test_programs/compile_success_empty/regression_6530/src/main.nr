struct Foo {
    inner: Field,
}

impl Into<Field> for Foo {
    fn into(self) -> Field {
        self.inner
    }
}

fn main() {
    let foo = Foo {
        inner: 0,
    };

    // This works:
    let _: Field = Into::<Field>::into(foo);

    // This was failing with 'No matching impl':
    let _: Field = foo.into();
}
