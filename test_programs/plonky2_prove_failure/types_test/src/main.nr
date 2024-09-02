struct Point {
    x: Field,
    y: Field
}

fn foo(
    a: Field,
    b: u32,
    c: Point,
    d: Field,
    e: i8,
    f: bool,
    g: str<11>,
    h: [Field; 2],
    i: &mut Field,
    j: fn() -> Field
) -> Field {
    a
        + b as Field
        + c.x
        + c.y
        + d
        + e as Field
        + f as Field
        + g.as_bytes()[10] as Field
        + h[0]
        + h[1]
        + *i
        + j()
}

fn bar() -> Field {
    11
}

fn main(a: Field, b: u32, c: Point, d: Field, e: i8, f: bool, g: str<11>, h: [Field; 2]) {
    let result = foo(a, b, c, d, e, f, g, h, &mut d, bar);
    assert(0x9f == result);
}
