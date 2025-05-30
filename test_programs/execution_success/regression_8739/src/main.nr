fn main() {
    // Safety: testing context
    unsafe {
        func_2()
    }
}
unconstrained fn func_2() {
<<<<<<< HEAD
    let mut a: [&mut bool; 1] = [&mut true];
    let mut idx_b: u32 = 0;
    while *a[0] {
        if idx_b == 0 {
=======
    let mut a: [&mut bool; 1] = [(&mut true)];
    let mut idx_b: u32 = 0;
    while (*a[0]) {
        if (idx_b == 0) {
>>>>>>> 76077f623e (keep track of references in makearray as to not remove them with last stores)
            break;
        }
    }
}
