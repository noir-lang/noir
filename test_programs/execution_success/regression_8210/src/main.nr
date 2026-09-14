// https://github.com/noir-lang/noir/issues/8210
//
// Accessing a trait's associated constant via the trait name (`Foo::BAR`) inside a default
// method body used to ICE during monomorphization. Referencing it via `Self::BAR` already
// worked; both forms should resolve to the implementing type's value.
trait Foo {
    let BAR: u32;

    fn via_trait(_: Self) -> u32 {
        Foo::BAR
    }

    fn via_self(_: Self) -> u32 {
        Self::BAR
    }

    fn arith(_: Self) -> u32 {
        Foo::BAR + 1
    }
}

struct Wrapper<T> {
    inner: T,
}

impl Foo for Field {
    let BAR: u32 = 254;
}

impl Foo for u8 {
    let BAR: u32 = 7;
}

impl<T> Foo for Wrapper<T> {
    let BAR: u32 = 11;
}

fn main() {
    assert((42 as Field).via_trait() == 254);
    assert((42 as Field).via_self() == 254);
    assert((42 as Field).arith() == 255);

    // A different primitive impl reached through the same default method must resolve to
    // its own value, not a single cached one.
    assert((0 as u8).via_trait() == 7);

    // The impl's self type may be generic.
    let w: Wrapper<Field> = Wrapper { inner: 0 };
    assert(w.via_trait() == 11);
}
