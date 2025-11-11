use acvm::AcirField;

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligContext,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Call the procedure that generates an error using the error selector
    /// This procedure is useful to deduplicate generating code for the same selector.
    pub(crate) fn call_error_with_string_procedure(&mut self, error_string: String) {
        self.add_procedure_call_instruction(ProcedureId::ErrorWithString(error_string));
    }
}

pub(super) fn compile_error_with_string_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    error_string: String,
) {
    brillig_context.error_with_string(error_string);
}
