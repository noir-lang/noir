use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligContext,
    brillig_variable::SingleAddrVariable,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Copy the input arguments to the [ScratchSpace], then emit an opcode to call [ProcedureId::MemCopy].
    ///
    /// Copies `num_elements_variable` number of items on the heap from `source_pointer` to `destination_pointer`.
    pub(crate) fn call_mem_copy_procedure(
        &mut self,
        source_pointer: MemoryAddress,
        destination_pointer: MemoryAddress,
        num_elements_variable: MemoryAddress,
    ) {
        let [source_pointer_arg, destination_pointer_arg, num_elements_variable_arg] =
            self.make_scratch_registers();
        self.mov_instruction(source_pointer_arg, source_pointer);
        self.mov_instruction(destination_pointer_arg, destination_pointer);
        self.mov_instruction(num_elements_variable_arg, num_elements_variable);
        self.add_procedure_call_instruction(ProcedureId::MemCopy);
    }
}

/// Compile [ProcedureId::MemCopy].
pub(super) fn compile_mem_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [source_pointer_arg, destination_pointer_arg, num_elements_variable_arg] =
        brillig_context.allocate_scratch_registers();

    brillig_context.codegen_mem_copy(
        source_pointer_arg,
        destination_pointer_arg,
        SingleAddrVariable::new_usize(num_elements_variable_arg),
    );
}
