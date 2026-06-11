// Assigning through a dereference whose operand is an arbitrary reference-producing expression
// (e.g. `*(&mut x)` or `*func()`), not just a place like `*x`.

struct S {
    y: Field,
}

fn id(r: &mut Field) -> &mut Field {
    r
}

fn main(x: Field) {
    let mut a = x;

    // Equivalent to `a = a + 10`, but written through an explicit reference.
    *(&mut a) = a + 10;

    // Compound assignment through an explicit reference reads and writes the same place.
    *(&mut a) += 5;

    // The reference operand can itself be a member access.
    let mut s = S { y: 0 };
    *(&mut s.y) = a + 10;
    *(&mut s.y) -= a;

    // The reference operand can be a function call that returns a reference. Writing through it
    // mutates the same value the returned reference points to.
    *id(&mut a) = s.y + 90;

    assert(a == s.y + 90);
    assert(s.y == 10);
}
