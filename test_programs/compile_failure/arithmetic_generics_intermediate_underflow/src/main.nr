// docs:start:intermediate-underflow-example
fn main() {
    // From main it looks like there's nothing sketchy going on
    seems_fine([]);
}

// Since `seems_fine` says it can receive and return any length N
fn seems_fine<let N: u32>(array: [Field; N]) -> [Field; N] {
    // But inside `seems_fine` we pop from the array which
    // requires the length to be greater than zero.

    // error: Could not determine array length `(0 - 1)`
    push_zero(pop(array))
}

fn pop<let N: u32>(array: [Field; N]) -> [Field; N - 1] {
    let mut result: [Field; N - 1] = std::mem::zeroed();
    for i in 0..N - 1 {
        result[i] = array[i];
    }
    result
}

fn push_zero<let N: u32>(array: [Field; N]) -> [Field; N + 1] {
    let mut result: [Field; N + 1] = std::mem::zeroed();
    for i in 0..N {
        result[i] = array[i];
    }
    // index N is already zeroed
    result
}
// docs:end:intermediate-underflow-example
