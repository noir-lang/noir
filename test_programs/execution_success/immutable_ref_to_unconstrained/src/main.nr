unconstrained fn read_ref(r: &Field) -> Field {
    *r
}

unconstrained fn read_ref_struct(r: &Foo) -> u32 {
    r.x + r.y[0] + r.y[1] + r.y[2]
}

struct Foo {
    x: u32,
    y: [u32; 3],
}

fn main(x: pub Field, y: pub u32) {
    // Safety:
    let result = unsafe { read_ref(&x) };
    assert_eq(result, x);

    let foo = Foo { x: 1, y: [2, 3, 4] };
    // Safety:
    let result = unsafe { read_ref_struct(&foo) };
    assert_eq(result, y);
}
