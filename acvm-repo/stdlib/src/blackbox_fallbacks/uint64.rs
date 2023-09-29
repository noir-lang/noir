use crate::impl_uint;

impl_uint!(UInt64, u64, 64);
impl UInt64 {
    /// Load a [UInt64] from eight [Witness]es each representing a [u8]
    pub(crate) fn from_witnesses(
        witnesses: &[Witness],
        mut num_witness: u32,
    ) -> (Vec<UInt64>, Vec<Opcode>, u32) {
        let mut new_opcodes = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let mut uint = Vec::new();

        for i in 0..witnesses.len() / 8 {
            let new_witness = variables.new_variable();
            uint.push(UInt64::new(new_witness));
            let mut expr = Expression::from(new_witness);
            for j in 0..8 {
                let scaling_factor_value: u128 = 1 << (8 * (7 - j) as u32);
                let scaling_factor = FieldElement::from(scaling_factor_value);
                expr.push_addition_term(-scaling_factor, witnesses[i * 8 + j]);
            }

            new_opcodes.push(Opcode::Arithmetic(expr));
        }
        let num_witness = variables.finalize();

        (uint, new_opcodes, num_witness)
    }
}
