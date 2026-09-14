// A dynamic-predicate read at a *constant* index into a vector whose item layout is
// heterogeneous must still collapse to a valid slot on a disabled branch.
//
// `S` flattens to `Field, u8, u8`, so a constant index is resolved by the element-type-sizes
// table into an already-in-bounds flat offset, which needs no predicate gating. Biasing that
// offset again by the fallback offset of the read field pushes the disabled-branch read past
// the value it should land on: with two items the block holds 6 slots and the read of `b`
// starts at slot 5, so its second leaf reads slot 6 and the whole circuit fails to solve,
// even though `c = false` means the read never happens.
struct S {
    a: Field,
    b: [u8; 2],
}

fn main(x: [S; 2], i: u32, c: bool, v: Field) -> pub [u8; 2] {
    let mut s: [S] = x.as_vector();
    // A write at a dynamic index moves the vector into a runtime memory block, so the read
    // below is issued as a memory operation rather than resolved at compile time.
    s[i] = S { a: v, b: [0, 0] };
    let mut r: [u8; 2] = [0, 0];
    if c {
        r = s[1].b;
    }
    r
}
