// Tests a very simple program.
// 
// The features being tested is basic arithmetics on brillig
fn main(x: u32, y: u32, result: u32)  {
    assert(result == add_int(x, y));
}

unconstrained fn add_int(x : u32, y : u32) -> u32 {
    x + y
}