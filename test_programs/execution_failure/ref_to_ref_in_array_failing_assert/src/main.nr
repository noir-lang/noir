// `write` sets the cell to `2 * y`, so this assertion is false for every `y != 0` and the
// circuit must not solve. `arr` holds both `p` and `q`, merging their alias classes.
fn write(q: &mut &mut Field, v: Field) {
    let p = *q;
    *p = v;
}

fn main(x: Field, y: Field) {
    let mut a: Field = x;
    let mut p = &mut a;
    let q = &mut p;
    let arr = [(p, q)];
    *p = y;
    write(q, y * 2);
    assert(*(arr[0].0) == y);
}
