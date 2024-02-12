// docs:start:pop_capsule
#[oracle(popCapsule)]
fn pop_capsule_oracle<N>() -> [Field; N] {}

// A capsule is a "blob" of data that is passed to the contract through an oracle.
unconstrained pub fn pop_capsule<N>() -> [Field; N] {
    pop_capsule_oracle()
}
// docs:end:pop_capsule

