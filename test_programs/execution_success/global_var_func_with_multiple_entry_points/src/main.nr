global ONE: Field = 1;
global TWO: Field = 2;
global THREE: Field = 3;

fn main(x: Field, y: pub Field) {
    /// Safety: testing context
    unsafe {
        entry_point_one(x, y);
        entry_point_two(x, y);
    }
}

unconstrained fn entry_point_one(x: Field, y: Field) {
    let z = ONE + x + y;
    assert(z == 2);
    inner_func(x, y);
}

unconstrained fn entry_point_two(x: Field, y: Field) {
    let z = TWO + x + y;
    assert(z == 3);
    inner_func(x, y);
}

unconstrained fn inner_func(x: Field, y: Field) {
    let z = THREE + x + y;
    assert(z == 4);
}