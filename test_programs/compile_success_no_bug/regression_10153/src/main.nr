pub fn main() -> pub Field {
    let _f = foo; // function pointer with #[fold]

    // Safety: test
    unsafe {
        bar()
    }
}

#[fold]
fn foo() -> Field {
    0
}

unconstrained fn bar() -> Field {
    foo() // #[fold] called from unconstrained
}
