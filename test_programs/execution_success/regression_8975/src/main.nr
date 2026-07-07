fn main() -> pub bool {
    // Safety:
    unsafe {
        func_1()
    }
    true
}
unconstrained fn func_1() {
    func_2([&mut true])
}
unconstrained fn func_2(mut b: [&mut bool; 1]) {
    b[0] = (&mut false);
    {
        let mut idx_c: u32 = 0;
        while (*b[0]) {
            if (idx_c == 0) {
                break
            }
        }
    };
}
