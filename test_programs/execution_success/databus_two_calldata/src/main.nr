// An simple program demonstrating two calldata array inputs and a single return data array. As an arbitrary example,
// the return data is computed as a linear combination of the calldata.
fn main(
    mut x: [u32; 4],
    y: call_data(0) [u32; 3],
    z: call_data(1) [u32; 4],
) -> return_data [u32; 4] {
    let mut result = [0; 4];
    for i in 0..3 {
        let idx = x[i];
        result[idx] = y[idx] + z[idx];
    }
    result[x[3]] = z[x[3]];
    result
}
