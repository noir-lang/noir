unconstrained fn main() {
    comptime {
        func_1(false)
    }
}

unconstrained fn func_1(terminate: bool) {
    if !terminate {
        let _ = -8_i8 >> -8_i8;
        func_1(true)
    }
}
