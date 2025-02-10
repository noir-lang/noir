fn main() {
    /// Safety: n/a
    unsafe { loophole() };
}


unconstrained fn loophole() {
    let mut i = 0;
    loop {
        println(i);
        i += 1;
        if false {
            break;
        }
    }
}