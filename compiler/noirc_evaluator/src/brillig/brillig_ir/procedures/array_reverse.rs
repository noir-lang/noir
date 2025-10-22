use std::vec;

use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::ProcedureId;
use crate::{
    allocate_scratch_registers,
    brillig::brillig_ir::{
        BrilligContext,
        debug_show::DebugToString,
        registers::{RegisterAllocator, ScratchSpace},
    },
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    pub(crate) fn call_array_reverse_procedure(
        &mut self,
        pointer: MemoryAddress,
        size: MemoryAddress,
    ) {
        let scratch_start = ScratchSpace::start();
        let source_pointer = MemoryAddress::direct(scratch_start);
        let size_register = MemoryAddress::direct(scratch_start + 1);

        self.mov_instruction(source_pointer, pointer);
        self.mov_instruction(size_register, size);

        self.add_procedure_call_instruction(ProcedureId::ArrayReverse);
    }
}

pub(super) fn compile_array_reverse_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    allocate_scratch_registers!(brillig_context, [source_pointer, size_register]);

    brillig_context.codegen_array_reverse(source_pointer, size_register);
}
