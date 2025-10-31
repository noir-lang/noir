use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligContext,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Copy the arguments to the scratch space and call [ProcedureId::ArrayReverse].
    ///
    /// Reverses the `size` number of items pointed to by `pointer` in-place.
    pub(crate) fn call_array_reverse_procedure(
        &mut self,
        pointer: MemoryAddress,
        size: MemoryAddress,
    ) {
        let [source_pointer_arg, size_register_arg] = self.make_scratch_registers();

        self.mov_instruction(source_pointer_arg, pointer);
        self.mov_instruction(size_register_arg, size);

        self.add_procedure_call_instruction(ProcedureId::ArrayReverse);
    }
}

/// Compile [ProcedureId::ArrayReverse].
pub(super) fn compile_array_reverse_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [source_pointer_arg, size_register_arg] = brillig_context.allocate_scratch_registers();

    brillig_context.codegen_array_reverse(source_pointer_arg, size_register_arg);
}
