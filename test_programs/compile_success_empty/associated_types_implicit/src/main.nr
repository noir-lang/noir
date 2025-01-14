trait Foo {
    type Bar;

    fn foo(self) -> Self::Bar;
}

impl Foo for u64 {
    type Bar = u8;

    fn foo(self) -> Self::Bar {
        self as u8
    }
}

fn main() {
    // This currently requires a type annotation to find the impl
    let three: u64 = 3;
    call_foo(three);

    let x: Option<Option<u64>> = Option::some(Option::some(0));
    let x_foo = x.foo();
    assert_eq(x_foo, x_foo); // ensure we don't need an additional type annotation for Bar here

    // The `as u8` is still necessary even though we know the object type,
    // otherwise we try to search for `u64: Foo<Bar = Field>`.
    // It seems the `Bar = u8` constraint is still there & checked for, but
    // the defaulting of the polymorphic integer occurs first.
    assert_eq(x.foo(), 0 as u8);
}

// Ensure we can use `<T as Foo>::Bar: Eq` in a function's where clause
fn call_foo<T>(x: T)
where
    T: Foo,
    <T as Foo>::Bar: Eq,
{
    let y = x.foo();
    assert_eq(y, y);
}

// Ensure we can use `<T as Foo>::Bar: Eq` in a trait impl's where clause
impl<T> Foo for Option<T>
where
    T: Foo,
    <T as Foo>::Bar: Eq,
{
    type Bar = <T as Foo>::Bar;

    fn foo(self) -> Self::Bar {
        self.unwrap().foo()
    }
}

// Ensure we can use `<T as Foo>::Bar: Eq` in a trait's where clause
// TODO: Not working, see issue #7024
// trait Baz<T> where
//     T: Foo,
//     <T as Foo>::Bar: Eq {}
//
// impl Baz<u64> for u32 {}
