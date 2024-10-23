struct Foo {
    x: u32,
    y: [u32; 10],
}

fn main(
    foos: call_data(0) [Foo; 2],
    values: call_data(0) [[[u32; 2]; 2]; 3],
    zero: u32,
    one: u32,
) -> pub u32 {
    assert_eq(foos[zero].x + 1, foos[one].x);
    assert_eq(foos[zero].y[3] + 2, foos[one].y[4]);
    assert_eq(values[zero][one][zero], values[one][zero][one]);
    foos[zero].x + foos[one].y[0]
}
