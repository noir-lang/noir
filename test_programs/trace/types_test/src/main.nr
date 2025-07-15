struct Point {
    x: Field,
    y: Field
}

fn main(a: Field, b: u32, c: Point, d: Field, e: i8, f: bool, g: str<11>, h: [Field; 2]) {
    let result = a
        + b as Field
        + c.x
        + c.y
        + d
        + (e as u8) as Field
        + f as Field
        + g.as_bytes()[10] as Field
        + h[0]
        + h[1];
    assert(0x91 == result);
}
