// First-class functions over mutable references. Defunctionalization must keep
// the `&mut` in the generated apply function's parameter and return types so
// that passes running afterwards still see which call arguments a callee may
// write through.
fn add1(r: &mut Field) -> &mut Field {
    *r += 1;
    r
}

fn mul2(r: &mut Field) -> &mut Field {
    *r *= 2;
    r
}

fn main(c: bool, x: Field) {
    let f = if c { add1 } else { mul2 };
    let mut v = x;
    let r = f(&mut v);
    *r += 10;
    let expected = if c { x + 11 } else { x * 2 + 10 };
    assert(v == expected);

    // Safety: same dispatch exercised in an unconstrained runtime.
    unsafe {
        dispatch_unconstrained(c, x);
    }
}

unconstrained fn dispatch_unconstrained(c: bool, x: Field) {
    let f = if c { mul2 } else { add1 };
    let mut v = x;
    let r = f(&mut v);
    *r += 10;
    let expected = if c { x * 2 + 10 } else { x + 11 };
    assert(v == expected);
}
