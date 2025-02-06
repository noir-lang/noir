global ONE: Field = 1;
global TWO: Field = 2;
global THREE: Field = 3;

fn main(x: Field, y: pub Field) {
    /// Safety: testing context
    unsafe {
        // entry_point_one(x, y);
        entry_point_two(x, y);
        entry_point_three(x, y);
        // entry_point_four(x, y);
    }
}

// unconstrained fn entry_point_one(x: Field, y: Field) {
//     let z = ONE + x + y;
//     assert(z == 2);
//     inner_func_one(x, y);
// }

unconstrained fn entry_point_two(x: Field, y: Field) {
    let z = TWO + x + y;
    assert(z == 3);
    inner_func_two(x, y);
}

// Identical to `entry_point_two`
unconstrained fn entry_point_three(x: Field, y: Field) {
    let z = TWO + x + y;
    assert(z == 3);
    inner_func_two(x, y);
}

// Identical to `entry_point_two`
// unconstrained fn entry_point_four(x: Field, y: Field) {
//     let z = TWO + x + y;
//     assert(z == 3);
//     inner_func_two(x, y);
// }

unconstrained fn inner_func_one(x: Field, y: Field) {
    let z = ONE + x + y;
    assert(z == 2);
}

unconstrained fn inner_func_two(x: Field, y: Field) {
    let z = TWO + x + y;
    assert(z == 3);
    nested_inner_func(x, y);
}

unconstrained fn nested_inner_func(x: Field, y: Field) {
    let z = THREE + x + y;
    assert(z == 4);
}
