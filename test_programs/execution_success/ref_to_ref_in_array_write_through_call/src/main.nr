// `arr` holds both `p` and `q`, where `q` points at the variable holding `p`.
// A field-insensitive alias analysis puts both references in one class, and the two
// `allocate`s keep distinct allocation sites — so `write` must still be seen to reach
// `*p` through `*q`, and the read after the call must observe the `2 * y` it wrote.
fn write(q: &mut &mut Field, v: Field) {
    let p = *q;
    *p = v;
}

fn main(x: Field, y: Field) -> pub Field {
    let mut a: Field = x;
    let mut p = &mut a;
    let q = &mut p;
    let arr = [(p, q)];
    *p = y;
    write(q, y * 2);
    *(arr[0].0)
}
