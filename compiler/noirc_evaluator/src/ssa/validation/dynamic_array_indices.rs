use crate::{
    errors::{RtResult, RuntimeError},
    ssa::{
        ir::{dfg::DataFlowGraph, function::Function, instruction::Instruction, value::ValueId},
        ssa_gen::Ssa,
    },
};

impl Ssa {
    /// Verifies there are no `array_get` or `array_set` instructions remaining
    /// with dynamic indices where the element type may contain a reference type.
    /// This effectively bans dynamic-indexing of arrays with reference elements
    /// since we cannot guarantee we optimize references out of the program in that case.
    ///
    /// This pass expects to be run late in the Ssa pipeline such that all array indices
    /// used are either constant or derived from an input to the program. E.g. we expect
    /// optimizations from inlining, flattening, and mem2reg to already be complete.
    pub(crate) fn verify_no_dynamic_indices_to_references(self) -> RtResult<Ssa> {
        for function in self.functions.values() {
            function.verify_no_dynamic_indices_to_references()?;
        }
        Ok(self)
    }
}

impl Function {
    pub(crate) fn verify_no_dynamic_indices_to_references(&self) -> RtResult<()> {
        if self.runtime().is_brillig() {
            return Ok(());
        }

        for block in self.reachable_blocks() {
            for instruction in self.dfg[block].instructions() {
                match &self.dfg[*instruction] {
                    Instruction::ArrayGet { array, index, .. }
                    | Instruction::ArraySet { array, index, .. } => {
                        let array_type = self.dfg.type_of_value(*array);
                        let contains_reference = array_type.contains_reference();

                        if contains_reference && !is_non_dynamic(&self.dfg, *index) {
                            let call_stack = self.dfg.get_instruction_call_stack(*instruction);
                            return Err(RuntimeError::DynamicIndexingWithReference { call_stack });
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(())
    }
}

/// Check if an value is a numeric constant, or a result of an instruction that only uses numeric constant inputs.
fn is_non_dynamic(dfg: &DataFlowGraph, value: ValueId) -> bool {
    // We could check if a non-constant-numeric value is a result of for example a binary an instruction that only
    // takes numeric constant input. However, if we have such a value, it might be a result of an overflowing
    // index expression that we could not simplify at runtime, which means most likely mem2reg could not eliminate
    // the reference allocation either, so even if we classified such indexes as non-dynamic, since they only use
    // known constants, we would just get another obscure error down the line with a less obvious call stack.
    dfg.get_numeric_constant(value).is_some()
}

#[cfg(test)]
mod tests {
    use crate::{errors::RuntimeError, ssa::ssa_gen::Ssa};

    #[test]
    fn dynamic_array_of_2_mut_bools() {
        // https://github.com/noir-lang/noir/issues/8750
        // fn main(c: u32) -> pub bool {
        //     let b: [&mut bool; 2] = [&mut false, &mut true];
        //     *b[c % 2]
        //}
        let src = r#"
            acir(inline) predicate_pure fn main f0 {
            b0(v0: u32):
                v1 = allocate -> &mut u1
                v2 = allocate -> &mut u1
                v3 = make_array [v1, v2] : [&mut u1; 2]
                v4 = truncate v0 to 1 bits, max_bit_size: 32
                v6 = lt v4, u32 2
                constrain v6 == u1 1, "Index out of bounds"
                v8 = array_get v3, index v4 -> &mut u1
                v9 = load v8 -> u1
                return v9
            }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = ssa.verify_no_dynamic_indices_to_references();
        assert!(matches!(result, Err(RuntimeError::DynamicIndexingWithReference { .. })));
    }

    #[test]
    fn no_error_in_brillig() {
        // unconstrained fn main(c: u32) -> pub bool {
        //     let b: [&mut bool; 2] = [&mut false, &mut true];
        //     *b[c % 2]
        //}
        let src = r#"
            brillig(inline) predicate_pure fn main f0 {
            b0(v0: u32):
                v1 = allocate -> &mut u1
                v2 = allocate -> &mut u1
                v3 = make_array [v1, v2] : [&mut u1; 2]
                v4 = truncate v0 to 1 bits, max_bit_size: 32
                v6 = lt v4, u32 2
                constrain v6 == u1 1, "Index out of bounds"
                v8 = array_get v3, index v4 -> &mut u1
                v9 = load v8 -> u1
                return v9
            }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = ssa.verify_no_dynamic_indices_to_references();
        assert!(result.is_ok());
    }

    #[test]
    fn error_on_index_overflow() {
        // https://github.com/noir-lang/noir/issues/9853
        // fn main() -> pub bool {
        //     let mut e: [(&mut Field, bool); 1] = [((&mut -1), false)];
        //     e[2147483648].1
        // }
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v2 = make_array [v0, u1 0] : [(&mut Field, u1); 1]
            v5 = unchecked_mul u32 2147483648, u32 2
            v7 = unchecked_add v5, u32 1
            v8 = array_get v2, index v7 -> u1
            return v8
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let result = ssa.verify_no_dynamic_indices_to_references();
        assert!(matches!(result, Err(RuntimeError::DynamicIndexingWithReference { .. })));
    }
}
