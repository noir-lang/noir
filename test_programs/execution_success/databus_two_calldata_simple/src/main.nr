fn main(mut idx: u32, y: call_data(0) [u32; 2], z: call_data(1) [u32; 4]) -> return_data u32 {
    let a = y[idx];
    let b = z[idx];
    a + b
}
