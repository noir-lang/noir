// A composite `array_get` under a false predicate reads the fallback flat slots
// `0, 1, 2, ...` of the element layout in order. Every leaf whose declared type is
// narrower than the fallback slot's real type must be multiplied by the predicate,
// otherwise `truncate` (and other range-carrying consumers) lay down constraints
// against a witness that is not bounded by its declared type.
//
// Here `Outer` begins with a `u8` tag but `Inner.b` is a `Field`. When `c = 0`,
// `items[1].a & 15` (which lowers to `truncate ... to 4 bits`) sees a leaf declared
// `u8` that was actually read from a slot holding `x[0].items[0].b` — an unconstrained
// `Field`. A wide value there must not make the disabled branch unsatisfiable.
struct Inner {
    a: u8,
    b: Field,
}

struct Outer {
    tag: u8,
    items: [Inner; 2],
}

fn main(x: [Outer; 3], i: u32, c: bool) -> pub u8 {
    let mut r: u8 = 0;
    if c {
        let items = x[i].items;
        r = items[1].a & 15;
    }
    r
}
