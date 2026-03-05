fn main() {
    // Safety: steps limited during constant folding.
    unsafe { func_2(false) };
}

unconstrained fn func_2(cond: bool) {
    loop {
        if cond {
            break
        }
    }
}
