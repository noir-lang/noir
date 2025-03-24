fn main() {
    let input: [u32; 2] = unsafe { inject_array() };

    let table: [u32; 2] = [0; 2];

    let mut s = table[1];
    let s_idx = s + input[0];
    s = table[s_idx];
    let s_idx = s + input[1];
    s = table[s_idx];
}

unconstrained fn inject_array() -> [u32; 2] {
    [0; 2]
}
