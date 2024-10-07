struct Foo {
    x: u32,
    y: u32,
}

fn main(foos: call_data(0) [Foo; 2], zero: u32, one: u32) -> return_data u32 {
    assert_eq(foos[zero].x + 1, foos[one].x);
    assert_eq(foos[zero].y + 2, foos[one].y);
    foos[zero].x + foos[one].y
}

