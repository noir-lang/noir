unconstrained fn uncon_ref() -> &mut Field {
    let lr = &mut 7;
    lr
}

fn main() {
    /// Safety: testing context
    let e = unsafe { uncon_ref() };
}
