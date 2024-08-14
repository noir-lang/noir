fn main(mut x: u32, y: call_data(0) u32, z: call_data(0) [u32; 4]) -> return_data u32 {
    let a = z[x];
    unsafe {
        a + foo(y)
    }
}

// Use an unconstrained function to force the compiler to avoid inlining
unconstrained fn foo(x: u32) -> u32 {
    x + 1
}

