//! This module implements the expansion of `vector_enumerate` intrinsic calls.
//!
//! The `vector_enumerate` intrinsic takes a vector and a closure, and calls the closure
//! for each element with (element, index) as arguments. This pass expands such calls
//! by creating direct calls to the closure with concrete element and index values.
//!
//! This optimization only applies:
//! - In ACIR runtime (not Brillig)
//! - When the vector length is a compile-time constant
//! - When the vector data can be traced to a constant array
//!
//! This pass should run after:
//! - The main inlining pass (so closures are already available)
//! - The as_vector_optimization pass (so vector lengths are constants)

use acvm::{AcirField, FieldElement};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Instruction, InstructionId, Intrinsic},
        types::NumericType,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Expand `vector_enumerate` calls in ACIR functions where the vector length is constant.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn expand_vector_enumerate(mut self) -> Self {
        for func in self.functions.values_mut() {
            // Only expand in ACIR functions
            if func.runtime().is_acir() {
                func.expand_vector_enumerate();
            }
        }
        self
    }
}

impl Function {
    /// Expand `vector_enumerate` calls where the vector length is a compile-time constant.
    pub(crate) fn expand_vector_enumerate(&mut self) {
        // Find the VectorEnumerate intrinsic if it exists in this function
        let Some(vector_enumerate) =
            self.dfg.get_intrinsic(Intrinsic::VectorEnumerate).copied()
        else {
            return;
        };

        // Collect all blocks to process (we need to collect first to avoid borrow checker issues)
        let blocks: Vec<BasicBlockId> = self.reachable_blocks().into_iter().collect();

        for block_id in blocks {
            // Get all instructions in the block
            let instructions: Vec<InstructionId> =
                self.dfg[block_id].instructions().to_vec();

            for instruction_id in instructions {
                // Clone the instruction data to avoid borrow checker issues
                let instruction = self.dfg[instruction_id].clone();

                // Check if this is a call to vector_enumerate
                let (func, arguments) = match &instruction {
                    Instruction::Call { func, arguments } => (*func, arguments.clone()),
                    _ => continue,
                };

                if func != vector_enumerate {
                    continue;
                }

                // Try to expand this enumerate call
                if let Some(new_instructions) =
                    self.try_expand_enumerate(instruction_id, &arguments)
                {
                    // Replace the enumerate call with the expanded instructions
                    let closure_func = arguments[2];
                    self.replace_instruction_with_multiple(
                        block_id,
                        instruction_id,
                        new_instructions,
                        closure_func,
                    );
                }
            }
        }
    }

    /// Try to expand a vector_enumerate call if the vector length is constant.
    /// Returns Some(instructions) if expansion is possible, None otherwise.
    fn try_expand_enumerate(
        &mut self,
        _instruction_id: InstructionId,
        arguments: &[ValueId],
    ) -> Option<Vec<Instruction>> {
        // vector_enumerate has 3 arguments: length, vector, closure
        if arguments.len() != 3 {
            return None;
        }

        let length_value = arguments[0];
        let vector_value = arguments[1];
        let closure_func = arguments[2];

        // Check if the vector length is a compile-time constant
        let length = self.dfg.get_numeric_constant(length_value)?;
        let length = length.to_u128() as usize;

        // The vector value might come from as_vector, so we need to trace back to find the array
        let (vector, element_types) = if let Some(v) = self.dfg.get_array_constant(vector_value) {
            // Vector is directly a constant array
            v
        } else {
            // If not directly a constant, check if it's the result of an as_vector call
            // Find the instruction that produced this vector value by scanning all instructions
            let mut found_array = None;
            for block_id in self.reachable_blocks() {
                for instruction_id in self.dfg[block_id].instructions() {
                    let results = self.dfg.instruction_results(*instruction_id);
                    // Check if this instruction produces the vector value
                    if results.contains(&vector_value) {
                        let instruction = &self.dfg[*instruction_id];

                        if let Instruction::Call { func, arguments } = instruction {
                            // Check if this is an as_vector call
                            if matches!(self.dfg[*func], Value::Intrinsic(Intrinsic::AsVector)) {
                                // The input to as_vector is arguments[0], which should be a constant array
                                if let Some(array_constant) = self.dfg.get_array_constant(arguments[0]) {
                                    found_array = Some(array_constant);
                                    break;
                                }
                            }
                        }
                    }
                }
                if found_array.is_some() {
                    break;
                }
            }

            found_array?
        };

        let element_size = element_types.element_size().to_usize();

        // Generate instructions to call the closure for each element
        let mut instructions = Vec::with_capacity(length);

        for i in 0..length {
            let element_offset = i * element_size;

            // Collect all values for this element (handles tuples/multi-value elements)
            let mut call_args = Vec::with_capacity(element_size + 1);
            for j in 0..element_size {
                call_args.push(vector[element_offset + j]);
            }

            // Add the index as the last argument
            let index = self
                .dfg
                .make_constant(FieldElement::from(i as u128), NumericType::unsigned(32));
            call_args.push(index);

            // Create a call instruction to the closure
            let call = Instruction::Call { func: closure_func, arguments: call_args };
            instructions.push(call);
        }

        Some(instructions)
    }

    /// Replace a single instruction with multiple instructions in the given block.
    fn replace_instruction_with_multiple(
        &mut self,
        block_id: BasicBlockId,
        old_instruction_id: InstructionId,
        new_instructions: Vec<Instruction>,
        closure_func: ValueId,
    ) {
        // Get the call stack from the old instruction
        let call_stack = self.dfg.get_instruction_call_stack_id(old_instruction_id);

        // Get the return types of the closure function for the Call instructions
        let return_types = match &self.dfg[closure_func] {
            Value::Function(_) => {
                // Closures in vector_enumerate typically return void
                vec![]
            }
            _ => vec![],
        };

        // Remove the old instruction from the block
        let block = &mut self.dfg[block_id];
        let instructions = block.instructions_mut();
        instructions.retain(|&id| id != old_instruction_id);

        // Insert the new instructions at the end of the block
        // (position doesn't matter for correctness since these are independent calls)
        for instruction in new_instructions {
            self.dfg.insert_instruction_and_results_without_simplification(
                instruction,
                block_id,
                Some(return_types.clone()),
                call_stack,
            );
        }

        // The old instruction is now orphaned and will be removed by dead instruction elimination
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;

    use super::Ssa;

    #[test]
    fn expand_vector_enumerate_with_constant_vector() {
        // Test expanding vector_enumerate with a known constant vector
        // Note: as_vector_optimization must run first to make the length constant
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v4, v5 = call as_vector(v2) -> (u32, [Field])
            call vector_enumerate(v4, v5, f1)
            return
        }
        acir(inline) fn closure f1 {
          b0(v0: Field, v1: u32):
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Run as_vector_optimization first (as the pipeline does)
        let ssa = ssa.as_vector_optimization();
        let ssa = ssa.expand_vector_enumerate();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v4, v5 = call as_vector(v2) -> (u32, [Field])
            call f1(Field 10, u32 0)
            call f1(Field 20, u32 1)
            call f1(Field 30, u32 2)
            return
        }
        acir(inline) fn closure f1 {
          b0(v0: Field, v1: u32):
            return
        }
        ");
    }

    #[test]
    fn does_not_expand_with_non_constant_vector() {
        // Test that we don't expand when the vector is not constant
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2, v3 = call as_vector(v0) -> (u32, [Field])
            call vector_enumerate(v2, v3, f1)
            return
        }
        acir(inline) fn closure f1 {
          b0(v0: Field, v1: u32):
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.as_vector_optimization();
        let ssa = ssa.expand_vector_enumerate();

        // Should remain unchanged
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            v2, v3 = call as_vector(v0) -> (u32, [Field])
            call vector_enumerate(v2, v3, f1)
            return
        }
        acir(inline) fn closure f1 {
          b0(v0: Field, v1: u32):
            return
        }
        ");
    }

    #[test]
    fn does_not_expand_in_brillig() {
        // Test that we don't expand in Brillig (unconstrained) functions
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v2 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v4, v5 = call as_vector(v2) -> (u32, [Field])
            call vector_enumerate(v4, v5, f1)
            return
        }
        brillig(inline) fn closure f1 {
          b0(v0: Field, v1: u32):
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.as_vector_optimization();
        let ssa = ssa.expand_vector_enumerate();

        // Should remain unchanged because it's brillig
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v2 = make_array [Field 10, Field 20, Field 30] : [Field; 3]
            v4, v5 = call as_vector(v2) -> (u32, [Field])
            call vector_enumerate(v4, v5, f1)
            return
        }
        brillig(inline) fn closure f1 {
          b0(v0: Field, v1: u32):
            return
        }
        ");
    }
}
