fn hello<let N: u32>(_array: [u1; N]) {
    for _ in 0..N {}
}

fn main() {
    let array: [u1; 2] = [0, 1];
    hello(array);
}
