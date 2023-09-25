use crate::impl_uint;

impl_uint!(UInt32, u32, 32);
impl UInt32 {
    /// Load a [UInt32] from four [Witness]es each representing a [u8]
    pub(crate) fn from_witnesses(
        witnesses: &[Witness],
        mut num_witness: u32,
    ) -> (Vec<UInt32>, Vec<Opcode>, u32) {
        let mut new_opcodes = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let mut uint = Vec::new();

        for i in 0..witnesses.len() / 4 {
            let new_witness = variables.new_variable();
            uint.push(UInt32::new(new_witness));
            let mut expr = Expression::from(new_witness);
            for j in 0..4 {
                let scaling_factor_value = 1 << (8 * (3 - j) as u32);
                let scaling_factor = FieldElement::from(scaling_factor_value as u128);
                expr.push_addition_term(-scaling_factor, witnesses[i * 4 + j]);
            }

            new_opcodes.push(Opcode::Arithmetic(expr));
        }
        let num_witness = variables.finalize();

        (uint, new_opcodes, num_witness)
    }
}
