// `*p` and `*(arr[0].0)` name the same cell, so the two outputs must agree.
fn write(q: &mut &mut Field, v: Field) {
    let p = *q;
    *p = v;
}

fn main(x: Field, y: Field) -> pub (Field, Field) {
    let mut a: Field = x;
    let mut p = &mut a;
    let q = &mut p;
    let arr = [(p, q)];
    *p = y;
    write(q, 99);
    (*p, *(arr[0].0))
}
