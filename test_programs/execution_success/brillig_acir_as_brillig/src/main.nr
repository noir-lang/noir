fn main(x: u32) {
    /// Safety: testing context
    unsafe {
        assert(entry_point(x) == 2);
        swap_entry_point(x, x + 1);
        assert(deep_entry_point(x) == 4);
    }
}

fn inner(x: u32) -> u32 {
    x + 1
}

unconstrained fn entry_point(x: u32) -> u32 {
    inner(x + 1)
}

fn swap(x: u32, y: u32) -> (u32, u32) {
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

fn level_3(x: u32) -> u32 {
    x + 1
}

fn level_2(x: u32) -> u32 {
    level_3(x + 1)
}

fn level_1(x: u32) -> u32 {
    level_2(x + 1)
}

unconstrained fn deep_entry_point(x: u32) -> u32 {
    level_1(x + 1)
}
