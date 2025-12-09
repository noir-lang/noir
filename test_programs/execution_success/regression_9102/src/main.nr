fn main() -> pub [bool; 2] {
    // safety: Test
    unsafe {
        func_1([[true, true], [true, false]])
    }
}

unconstrained fn func_1(mut a: [[bool; 2]; 2]) -> [bool; 2] {
    let mut i: u32 = 1;
    loop {
        if (i == 1) {
            break
        }
    }
    a[1][1] = true;
    [true, false]
}
