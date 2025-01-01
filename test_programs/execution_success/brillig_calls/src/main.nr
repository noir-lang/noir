// Tests a very simple program.
//
// The features being tested is brillig calls
fn main(x: u32) {
    unsafe {
        /*@safety : testing context*/
        assert(entry_point(x) == 2);
        swap_entry_point(x, x + 1);
        assert(deep_entry_point(x) == 4);
        multiple_values_entry_point(x);
    }
}

unconstrained fn returns_multiple_values(x: u32) -> (u32, u32, u32, u32) {
    (x + 1, x + 2, x + 3, x + 4)
}

unconstrained fn multiple_values_entry_point(x: u32) {
    let (a, b, c, d) = returns_multiple_values(x);
    assert(a == x + 1);
    assert(b == x + 2);
    assert(c == x + 3);
    assert(d == x + 4);
}

unconstrained fn inner(x: u32) -> u32 {
    x + 1
}

unconstrained fn entry_point(x: u32) -> u32 {
    inner(x + 1)
}

unconstrained fn swap(x: u32, y: u32) -> (u32, u32) {
    (y, x)
}

unconstrained fn swap_entry_point(x: u32, y: u32) {
    let swapped = swap(x, y);
    assert(swapped.0 == y);
    assert(swapped.1 == x);
    let swapped_twice = swap(swapped.0, swapped.1);
    assert(swapped_twice.0 == x);
    assert(swapped_twice.1 == y);
}

unconstrained fn level_3(x: u32) -> u32 {
    x + 1
}

unconstrained fn level_2(x: u32) -> u32 {
    level_3(x + 1)
}

unconstrained fn level_1(x: u32) -> u32 {
    level_2(x + 1)
}

unconstrained fn deep_entry_point(x: u32) -> u32 {
    level_1(x + 1)
}
