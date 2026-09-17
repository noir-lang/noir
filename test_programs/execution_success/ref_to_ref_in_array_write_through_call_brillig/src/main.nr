unconstrained fn write(q: &mut &mut Field, v: Field) {
    let p = *q;
    *p = v;
}

unconstrained fn main(x: Field, y: Field) -> pub Field {
    let mut a: Field = x;
    let mut p = &mut a;
    let q = &mut p;
    let arr = [(p, q)];
    *p = y;
    write(q, y * 2);
    *(arr[0].0)
}
