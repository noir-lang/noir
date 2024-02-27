// Copied from noir-contracts/contracts/slow_tree_contract/src/capsule.nr
// We should extract this to a shared lib in aztec-nr once we settle on a design for capsules

#[oracle(popCapsule)]
fn pop_capsule_oracle<N>() -> [Field; N] {}

// A capsule is a "blob" of data that is passed to the contract through an oracle.
unconstrained pub fn pop_capsule<N>() -> [Field; N] {
    pop_capsule_oracle()
}
