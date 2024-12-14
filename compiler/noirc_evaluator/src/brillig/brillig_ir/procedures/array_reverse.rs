use std::vec;

use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
    BrilligContext,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    pub(crate) fn call_array_reverse_procedure(
        &mut self,
        pointer: MemoryAddress,
        size: MemoryAddress,
    ) {
        let source_pointer = MemoryAddress::direct(ScratchSpace::start());
        let size_register = MemoryAddress::direct(ScratchSpace::start() + 1);

        self.mov_instruction(source_pointer, pointer);
        self.mov_instruction(size_register, size);

        self.add_procedure_call_instruction(ProcedureId::ArrayReverse);
    }
}

pub(super) fn compile_array_reverse_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let source_pointer = MemoryAddress::direct(ScratchSpace::start());
    let size_register = MemoryAddress::direct(ScratchSpace::start() + 1);

    brillig_context.set_allocated_registers(vec![source_pointer, size_register]);

    brillig_context.codegen_array_reverse(source_pointer, size_register);
}
