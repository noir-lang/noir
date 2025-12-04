use acvm::{AcirField, acir::brillig::HeapVector};

use super::ProcedureId;
use crate::{
    brillig::brillig_ir::{
        BrilligContext, ReservedRegisters,
        debug_show::DebugToString,
        registers::{RegisterAllocator, ScratchSpace},
    },
    ssa::ir::instruction::ErrorType,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Reverts with the error selector of the given string
    /// This procedure is useful to deduplicate generating code for the same selector.
    pub(crate) fn call_error_with_string_procedure(&mut self, error_string: String) {
        self.add_procedure_call_instruction(ProcedureId::RevertWithString(error_string));
    }
}

pub(super) fn compile_error_with_string_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    error_string: String,
) {
    // Compute the error selector and register the error type
    let error_type = ErrorType::String(error_string);
    let error_selector = error_type.selector();
    brillig_context.obj.error_types.insert(error_selector, error_type);

    // Write the selector to the free memory pointer and trap with it as revert data
    brillig_context.indirect_const_instruction(
        ReservedRegisters::free_memory_pointer(),
        64,
        u128::from(error_selector.as_u64()).into(),
    );
    brillig_context.trap_instruction(HeapVector {
        pointer: ReservedRegisters::free_memory_pointer(),
        size: ReservedRegisters::usize_one(),
    });
}
