pub struct Foo<let Y: u32, A> {}

pub trait Bar {
    fn bar() -> Field;
}

impl<let X: u32> Bar for Foo<X, u8> {
    fn bar() -> Field {
        1
    }
}

impl<let X: u32> Bar for Foo<X, u16> {
    fn bar() -> Field {
        2
    }
}

fn g<let X_: u32>() -> Field {
    Foo::<X_, u8>::bar()
}

fn h<let X_: u32>() -> Field {
    Foo::<X_, u16>::bar()
}

fn main() {
    assert_eq(g::<0>(), 1);
    assert_eq(h::<0>(), 2);
}
