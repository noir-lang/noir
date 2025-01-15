fn main(args: [Field; 2]) {
    /// Safety: n/a
    unsafe { store(args) };
}

pub unconstrained fn store(_: [Field]) {}