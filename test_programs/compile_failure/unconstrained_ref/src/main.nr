unconstrained fn uncon_ref() -> &mut field {
    let lr = &mut 7;
    lr
}

fn main() {
    let e = uncon_ref();
}
