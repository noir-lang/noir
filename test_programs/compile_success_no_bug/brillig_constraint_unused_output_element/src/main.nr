unconstrained fn hints_helper(x: u32) -> [u32; 2] {
    [0, x]
}

fn main(x: u32) -> pub u32 {
    // Safety: hints[1] is constrained below; hints[0] is unused
    let hints = unsafe { hints_helper(x) };
    assert(hints[1] == x);
    hints[1]
}
