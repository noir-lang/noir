// A constant out-of-bounds index into a known-length array in an unconstrained (Brillig)
// function must fail with "Index out of bounds" rather than letting the Brillig VM grow its
// memory to fit the out-of-bounds offset (see issue #10035).
unconstrained fn main(value: u32) -> pub [u32; 4] {
    let mut array: [u32; 4] = [value; 4];
    array[index()] = value;
    array
}

// Indirection so the index is a compile-time constant in SSA without being a literal array index
// that the frontend would reject up front.
fn index() -> u32 {
    1000000000
}
