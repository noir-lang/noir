fn main(mut x: u32, y: u32, z: u32) {
    x = unsafe { x + y };
    assert(
        unsafe {
        x == z
    }
    );

    unsafe {
        x *= 8;
        assert(x > 9);
    }

    {}

    {}

    unsafe {}
    unsafe {}
}
