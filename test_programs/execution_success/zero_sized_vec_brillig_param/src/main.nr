// Regression: gen_brillig_parameters underflow when a vector with zero-sized
// elements has its length parameter pruned by dead-parameter elimination,
// leaving the vector as the first (or only) argument in a brillig call.

unconstrained fn takes_zero_sized_vec(b: [[u16; 0]]) -> u32 {
    b.len()
}

fn main() -> pub u32 {
    let v: [[u16; 0]] = &[[], [], []];
    // Safety: calling unconstrained function for testing
    unsafe { takes_zero_sized_vec(v) }
}
