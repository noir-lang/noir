#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Witness(pub u32);

impl Witness {
    pub fn new(witness_index: u32) -> Witness {
        Witness(witness_index)
    }
    pub fn witness_index(&self) -> u32 {
        self.0
    }
    pub fn as_usize(&self) -> usize {
        // This is safe as long as the architecture is 32bits minimum
        self.0 as usize
    }

    pub const fn can_defer_constraint(&self) -> bool {
        true
    }

    pub fn to_unknown(self) -> UnknownWitness {
        UnknownWitness(self.0)
    }
}

// This is a witness that is unknown relative to the rest of the witnesses in the arithmetic gate
// We use this, so that they are pushed to the beginning of the array
//
// When they are pushed to the beginning of the array, they are less likely to be used in an intermediate gate
// by the optimiser, which would mean two unknowns in an equation.
// See Issue #109
pub struct UnknownWitness(pub u32);

impl UnknownWitness {
    pub fn as_witness(&self) -> Witness {
        Witness(self.0)
    }
}
