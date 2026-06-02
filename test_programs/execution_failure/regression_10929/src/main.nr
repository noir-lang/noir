fn main(b: pub bool) {
    if (
        // safety:
        unsafe { func_3() >= func_3() }
    ) {
        assert(
            (
                if b {
                    // safety:
                    unsafe {
                        func_3()
                    }
                } else {
                    0_u128
                }
                    != 0_u128
            ),
            "QPA",
        );
    }
}
unconstrained fn func_3() -> u128 {
    0
}
