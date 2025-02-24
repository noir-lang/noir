// docs:start:underflow-example
fn pop<let N: u32>(array: [Field; N]) -> [Field; N - 1] {
    let mut result: [Field; N - 1] = std::mem::zeroed();
    for i in 0..N - 1 {
        result[i] = array[i];
    }
    result
}

fn main() {
    // error: Could not determine array length `(0 - 1)`
    pop([]);
}
// docs:end:underflow-example
