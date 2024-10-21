struct foo {
    value: Field,
    counter: u8,
    dummy: u8,
}
struct bar {
    dummy: [u8; 3],
    value: Field,
    counter: u8,
}
struct bar_field {
    dummy: [Field; 3],
    value: Field,
    counter: u8,
}
fn main(x: [foo; 3], y: u32, z: u32) -> pub u8 {
    let a = [y, z, x[y].counter as u32];
    let mut b = [bar { value: 0, counter: 0, dummy: [0; 3] }; 3];
    let mut c = [bar_field { value: 0, counter: 0, dummy: [0; 3] }; 3];
    for i in 0..3 {
        b[i].value = x[i].value;
        b[i].counter = x[i].counter;
        b[i].dummy[0] = x[i].dummy;
        c[i].value = x[i].value;
        c[i].counter = x[i].counter;
        c[i].dummy[0] = x[i].dummy as Field;
    }
    if z == 0 {
        // offset
        assert(y as u8 < x[y].counter);
        assert(y <= a[y]);
        // first element is compatible
        assert(y as u8 < b[y].counter);
        // fallback
        assert(y as u8 < c[y].counter);
    }
    x[0].counter
}
