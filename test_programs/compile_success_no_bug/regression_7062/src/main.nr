fn main(args: [Field; 2]) {
    /// Safety: n/a
    unsafe { store(args) };
    // Dummy test to remove the 'underconstraint bug'
    assert(args[0] + args[1] != 0);
}

pub unconstrained fn store(_: [Field]) {}