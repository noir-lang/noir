unconstrained fn uncon_ref() -> &mut Field {
    let lr = &mut 7;
    lr
}

fn main() {
    unsafe {
        let e = uncon_ref();
    }
}


