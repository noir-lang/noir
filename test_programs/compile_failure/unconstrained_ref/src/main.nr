unconstrained fn uncon_ref() -> &mut Field {
    let lr = &mut 7;
    lr
}

fn main() {
    let e = unsafe {
        //@safety: testing context
        uncon_ref()
    };
}
