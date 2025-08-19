fn main() {
    unsafe { func_2() };
}

unconstrained fn func_2() {
    loop {
        if false {
            break
        }
    }
}
