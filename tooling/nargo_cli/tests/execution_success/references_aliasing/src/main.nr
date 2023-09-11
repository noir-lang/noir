fn main() {
    let mut x = 100;
    let mut xref = &mut x;
    increment(xref);
    assert(*xref == 101);

    regression_2445();
}

fn increment(mut r: &mut Field) {
    *r = *r + 1;
}

// If aliasing within arrays and constant folding within the mem2reg pass aren't
// handled, we'll fail to optimize out all the references in this function.
fn regression_2445() {
    let mut var = 0;
    let ref = &mut &mut var;

    let mut array = [ref, ref];

    **array[0] = 1;
    **array[1] = 2;

    assert(var == 2);
    assert(**ref == 2);
    assert(**array[0] == 2);
    assert(**array[1] == 2);
}
