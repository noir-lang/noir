struct foo {
    value: Field,
    counter: u8,
    dummy: u8,
}
fn main(x: [foo; 3], y: u32, z: u32) -> pub u8 {
    if z == 0 {
        assert(y as u8 < x[y].counter);
    }
    x[0].counter
}
