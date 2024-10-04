struct Foo {
    x: u32,
    y: u32,
}

fn main(foos: call_data(0) [Foo; 2]) -> return_data u32 {
    assert_eq(foos[0].x + 1, foos[1].x);
    assert_eq(foos[0].y + 2, foos[1].y);
    foos[0].x + foos[0].y
}

