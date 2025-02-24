use acvm::AcirField;

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
    BrilligContext,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Reverts with the error selector of the given string
    /// This procedure is useful to deduplicate generating code for the same selector.
    pub(crate) fn call_revert_with_string_procedure(&mut self, revert_string: String) {
        self.add_procedure_call_instruction(ProcedureId::RevertWithString(revert_string));
    }
}

pub(super) fn compile_revert_with_string_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    revert_string: String,
) {
    brillig_context.revert_with_string(revert_string);
}
