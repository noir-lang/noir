use acir::native_types::Witness;

// Simple helper struct to keep track of the current witness index
// and create variables
pub struct VariableStore<'a> {
    witness_index: &'a mut u32,
}

impl<'a> VariableStore<'a> {
    pub fn new(witness_index: &'a mut u32) -> Self {
        Self { witness_index }
    }

    pub fn new_variable(&mut self) -> Witness {
        let witness = Witness(*self.witness_index);
        *self.witness_index += 1;
        witness
    }

    pub fn finalize(self) -> u32 {
        *self.witness_index
    }
}
