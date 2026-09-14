// `black_box` keeps the body from being recognized as a pure pass-through, so the
// ACIR→Brillig call survives optimization and the immutable reference is actually read.
unconstrained fn read_ref(r: &Field) -> Field {
    std::hint::black_box(*r)
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

    // `foo` is built from a witness so the compiler cannot fold the read away at
    // compile time, forcing the struct to be read back through the reference at runtime.
    let foo = Foo { x: y, y: [y, y, y] };
    // Safety:
    let result = unsafe { read_ref_struct(&foo) };
    assert_eq(result, y * 4);
}
