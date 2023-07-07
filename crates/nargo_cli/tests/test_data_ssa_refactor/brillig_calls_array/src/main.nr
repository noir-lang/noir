// Tests a very simple program.
// 
// The features being tested is brillig calls passing arrays around
fn main(x: [u32; 3])  {
    assert(entry_point(x) == 9);
}

unconstrained fn inner(x : [u32; 3]) -> [u32; 3] {
    [x[0] + 1, x[1] + 1, x[2] + 1]
}

unconstrained fn entry_point(x : [u32; 3]) -> u32 {
    let y = inner(x);
    y[0] + y[1] + y[2]
}

