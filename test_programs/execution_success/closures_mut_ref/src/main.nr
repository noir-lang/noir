fn main(mut x: Field) {
    let one = 1;
    let add1 = |z| { *z = *z + one; };

    let two = 2;
    let add2 = |z| { *z = *z + two; };

    add1(&mut x);
    assert(x == 1);

    add2(&mut x);
    assert(x == 3);

    issue_2120();
}
// https://github.com/noir-lang/noir/issues/2120
fn issue_2120() {
    let x1 = &mut 42;
    let set_x1 = |y| { *x1 = y; };

    assert(*x1 == 42);
    set_x1(44);
    assert(*x1 == 44);
    set_x1(*x1);
    assert(*x1 == 44);
}
