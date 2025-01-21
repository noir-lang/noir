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
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn evaluate_static_assert_and_assert_constant(
        mut self,
    ) -> Result<Ssa, RuntimeError> {
        for function in self.functions.values_mut() {
            function.evaluate_static_assert_and_assert_constant()?;
        }
        Ok(self)
    }
}

impl Function {
    pub(crate) fn evaluate_static_assert_and_assert_constant(
        &mut self,
    ) -> Result<(), RuntimeError> {
        for block in self.reachable_blocks() {
            // Unfortunately we can't just use instructions.retain(...) here since
            // check_instruction can also return an error
            let instructions = self.dfg[block].take_instructions();
            let mut filtered_instructions = Vec::with_capacity(instructions.len());

            for instruction in instructions {
                if check_instruction(self, instruction)? {
                    filtered_instructions.push(instruction);
                }
            }

            *self.dfg[block].instructions_mut() = filtered_instructions;
        }
        Ok(())
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
    let static_assert_id = function.dfg.import_intrinsic(Intrinsic::StaticAssert);
    match &function.dfg[instruction] {
        Instruction::Call { func, arguments } => {
            if *func == assert_constant_id {
                evaluate_assert_constant(function, instruction, arguments)
            } else if *func == static_assert_id {
                evaluate_static_assert(function, instruction, arguments)
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
        let call_stack = function.dfg.get_instruction_call_stack(instruction);
        Err(RuntimeError::AssertConstantFailed { call_stack })
    }
}

/// Evaluate a call to `static_assert`, returning an error if the value is false
/// or not constant (see assert_constant).
///
/// When it passes, Ok(false) is returned. This signifies a
/// success but also that the instruction need not be reinserted into the block being unrolled
/// since it has already been evaluated.
fn evaluate_static_assert(
    function: &Function,
    instruction: InstructionId,
    arguments: &[ValueId],
) -> Result<bool, RuntimeError> {
    if arguments.len() != 2 {
        panic!("ICE: static_assert called with wrong number of arguments")
    }

    if !function.dfg.is_constant(arguments[1]) {
        let call_stack = function.dfg.get_instruction_call_stack(instruction);
        return Err(RuntimeError::StaticAssertDynamicMessage { call_stack });
    }

    if function.dfg.is_constant_true(arguments[0]) {
        Ok(false)
    } else {
        let call_stack = function.dfg.get_instruction_call_stack(instruction);
        if function.dfg.is_constant(arguments[0]) {
            let message = function
                .dfg
                .get_string(arguments[1])
                .expect("Expected second argument to be a string");
            Err(RuntimeError::StaticAssertFailed { message, call_stack })
        } else {
            Err(RuntimeError::StaticAssertDynamicPredicate { call_stack })
        }
    }
}
