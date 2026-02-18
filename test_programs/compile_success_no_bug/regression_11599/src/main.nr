fn main() {
    // Safety: fuzzer repro
    unsafe { func(true) };
}
unconstrained fn func(b: bool) {
    for i in 0_u8..=(1_u8 % if b { 1_u8 } else { 1_u8 }) {
        println(i);
    }
}
