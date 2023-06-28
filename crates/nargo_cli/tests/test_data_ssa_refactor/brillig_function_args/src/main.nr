// Tests a very simple program.
// 
// The features being tested is brillig calls
fn main(x: u32) -> pub u32  {
    test(x)
}

unconstrained fn test(x: u32) -> u32 {
    wrapper(x, incement) +  wrapper(x, decrement)
}

unconstrained fn incement(x: u32) -> u32 {
    x + 1
}

unconstrained fn decrement(x: u32) -> u32 {
    x - 1
}

unconstrained fn wrapper(x: u32, f: fn(u32) -> u32) -> u32 {
    f(x)
}

