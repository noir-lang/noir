fn main(mut x: u32, y: pub u32) {
    let x_ref = &mut x;
    if x == 5  {
        alias_input(x_ref, y);
    }
    assert(x == 10);
}

unconstrained fn alias_input(x: &mut u32, y: u32) {
    *x = y;
}
