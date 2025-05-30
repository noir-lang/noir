fn main() {
    // Safety:
    let x = unsafe { func_1() };
    assert(x);
}

unconstrained fn func_1() -> bool {
    let mut x = true;
    loop {
        if true {
            break;
        }
        x = false;
    }
    x
}
