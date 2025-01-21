fn main(value: Field, index: u32) {
    let mut args = &[0, 1];
    args[index] = value;
    /// Safety: n/a
    unsafe { store(args) };
    // Dummy test to remove the 'underconstraint bug'
    assert(args[0] + args[1] != 0);
}

pub unconstrained fn store(_: [Field]) {}
