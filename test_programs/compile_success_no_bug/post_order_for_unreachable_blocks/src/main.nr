fn main() {
    // Safety: test
    unsafe { func_2(true) };
}

unconstrained fn func_2(cond: bool) {
    loop {
        if cond {
            break;
        }
        loop {
            if false {
                break
            }
        }
    }
}
