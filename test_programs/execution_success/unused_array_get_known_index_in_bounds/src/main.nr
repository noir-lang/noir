fn main() {
    // Safety:
    let result = unsafe { foo([0]) };
    println(result);
}

unconstrained fn foo(bytes: [u8; 1]) -> Field {
    for _ in 0..bytes.len() {
        let _ = bytes[0];
    }
    0
}
