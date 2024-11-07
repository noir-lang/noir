fn main() {
    unsafe { foo() };
}

unconstrained fn foo() {
    for i in 0..1 {
        if i == 0 {
            break;
        }
    }
}
