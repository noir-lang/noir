// Tests a very simple program.
// 
// The features being tested is brillig calls
fn main(x: u32)  {
    assert(entry_point(x) == 2);
}

unconstrained fn inner(x : u32) -> u32 {
    x + 1
}

unconstrained fn entry_point(x : u32) -> u32 {
    inner(x + 1)
}