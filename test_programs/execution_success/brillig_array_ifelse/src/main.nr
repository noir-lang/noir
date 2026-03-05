// Regression test for array IfElse mutation in Brillig.
fn main(cond: bool) -> pub (bool, [bool; 1], [bool; 1]) {
    // Safety: Brillig execution for testing
    unsafe {
        test_tuple_aliasing(cond)
    }
}

unconstrained fn test_tuple_aliasing(cond: bool) -> (bool, [bool; 1], [bool; 1]) {
    let mut a: (bool, [bool; 1], [bool; 1]) = (false, [false], [false]);
    // Same array a.1 is used twice in tuple to create aliasing
    a = if cond {
        (a.1[0], a.1, a.1)
    } else {
        (true, [true], [true])
    };

    // Mutate a.1
    a.1[0] = {
        a = if a.0 {
            (a.1[0], a.1, a.1)
        } else {
            (true, [true], [true])
        };
        false
    };
    // Result should be: (true, [false], [true])
    assert(a.1[0] != a.2[0]);
    a
}
