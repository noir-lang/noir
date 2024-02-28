
fn array_zeros<N?>(_x: [u64; N]) -> [u64; N] {
    [0; N]
}

fn main() {
    let xs: [u64] = [11111; 1234567]; 
    assert(array_zeros(xs) == xs);
}

