fn main() {
    // Safety: Test program
    let input: u32 = unsafe { inject_value() };

    let table: [u32; 2] = [0; 2];

    let mut s = table[1];
    s = table[s + input];
    s = table[s];
}

unconstrained fn inject_value() -> u32 {
    0
}
