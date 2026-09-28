//! `T: Foo<u8>` and `T: Foo<u32>` are distinct bounds on one trait, each with its own `Assoc`.
//! A projection through either must read the associated type that bound states, so
//! `<T as Foo<u8>>::Assoc` is `u8` and dispatches to `impl Bar for u8`.

trait Foo<A> {
    type Assoc;
}

trait Bar {
    fn tag() -> Field;
}

impl Bar for u8 {
    fn tag() -> Field {
        1
    }
}

impl Bar for u32 {
    fn tag() -> Field {
        2
    }
}

trait M8: Foo<u8, Assoc = u8> {}
trait M32a: Foo<u32, Assoc = u32> {}
trait M32b: Foo<u32, Assoc = u32> {}

struct S {}

impl Foo<u8> for S {
    type Assoc = u8;
}

impl Foo<u32> for S {
    type Assoc = u32;
}

impl M8 for S {}
impl M32a for S {}
impl M32b for S {}

fn tag_of_u8_assoc<T>() -> Field
where
    T: M8,
    T: M32a,
    T: M32b,
{
    <<T as Foo<u8>>::Assoc as Bar>::tag()
}

fn main(x: Field) {
    assert_eq(tag_of_u8_assoc::<S>() + x, 1);
}
