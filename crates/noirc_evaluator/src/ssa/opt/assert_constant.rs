use crate::{
    errors::RuntimeError,
    ssa::{
        ir::{
            function::Function,
            instruction::{Instruction, InstructionId, Intrinsic},
            value::ValueId,
        },
        ssa_gen::Ssa,
    },
};

impl Ssa {
    /// A simple SSA pass to go through each instruction and evaluate each call
    /// to `assert_constant`, issuing an error if any arguments to the function are
    /// not constants.
    ///
    /// Note that this pass must be placed directly before loop unrolling to be
    /// useful. Any optimization passes between this and loop unrolling will cause
    /// the constants that this pass sees to be potentially different than the constants
    /// seen by loop unrolling. Furthermore, this pass cannot be a part of loop unrolling
    /// since we must go through every instruction to find all references to `assert_constant`
    /// while loop unrolling only touches blocks with loops in them.
    pub(crate) fn evaluate_assert_constant(mut self) -> Result<Ssa, RuntimeError> {
        for function in self.functions.values_mut() {
            for block in function.reachable_blocks() {
                // Unfortunately we can't just use instructions.retain(...) here since
                // check_instruction can also return an error
                let instructions = function.dfg[block].take_instructions();
                let mut filtered_instructions = Vec::with_capacity(instructions.len());

                for instruction in instructions {
                    if check_instruction(function, instruction)? {
                        filtered_instructions.push(instruction);
                    }
                }

                *function.dfg[block].instructions_mut() = filtered_instructions;
            }
        }
        Ok(self)
    }
}

/// During the loop unrolling pass we also evaluate calls to `assert_constant`.
/// This is done in this pass because loop unrolling is the only pass that will error
/// if a value (the loop bounds) are not known constants.
///
/// This returns Ok(true) if the given instruction should be kept in the block and
/// Ok(false) if it should be removed.
fn check_instruction(
    function: &mut Function,
    instruction: InstructionId,
) -> Result<bool, RuntimeError> {
    let assert_constant_id = function.dfg.import_intrinsic(Intrinsic::AssertConstant);
    match &function.dfg[instruction] {
        Instruction::Call { func, arguments } => {
            if *func == assert_constant_id {
                evaluate_assert_constant(function, instruction, arguments)
            } else {
                Ok(true)
            }
        }
        _ => Ok(true),
    }
}

/// Evaluate a call to `assert_constant`, returning an error if any of the elements are not
/// constants. If all of the elements are constants, Ok(false) is returned. This signifies a
/// success but also that the instruction need not be reinserted into the block being unrolled
/// since it has already been evaluated.
fn evaluate_assert_constant(
    function: &Function,
    instruction: InstructionId,
    arguments: &[ValueId],
) -> Result<bool, RuntimeError> {
    if arguments.iter().all(|arg| function.dfg.is_constant(*arg)) {
        Ok(false)
    } else {
        let call_stack = function.dfg.get_call_stack(instruction);
        Err(RuntimeError::AssertConstantFailed { call_stack })
    }
}
