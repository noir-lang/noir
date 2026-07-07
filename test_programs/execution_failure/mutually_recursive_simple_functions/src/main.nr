fn main() {
    // Safety: testing context
    unsafe {
        starter();
    }
}

unconstrained fn starter() {
    ping();
}

unconstrained fn ping() {
    pong()
}

unconstrained fn pong() {
    ping()
}
