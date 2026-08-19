// The disabled-branch fallback index for a dynamic array read must be a flat
// slot offset, not an item ordinal. `Foo.b` is item 1 of the element but flat
// slot 2 (`a` occupies two slots): a fallback biased to flat slot 1 would read
// `a[1]` — an unconstrained `Field` — into a leaf tagged `u8`, and `& 15`
// (a 4-bit truncate) would become unsatisfiable even though `c = false` means
// the branch never runs.
struct Foo {
    a: [Field; 2],
    b: u8,
}

fn main(x: [Foo; 3], i: u32, c: bool) -> pub u8 {
    let mut r: u8 = 0;
    if c {
        r = x[i].b & 15;
    }
    r
}
