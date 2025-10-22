use std::vec;

use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::ProcedureId;
use crate::{
    allocate_scratch_registers,
    brillig::brillig_ir::{
        BrilligContext,
        brillig_variable::SingleAddrVariable,
        debug_show::DebugToString,
        registers::{RegisterAllocator, ScratchSpace},
    },
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    pub(crate) fn call_mem_copy_procedure(
        &mut self,
        source_pointer: MemoryAddress,
        destination_pointer: MemoryAddress,
        num_elements_variable: MemoryAddress,
    ) {
        let scratch_start = ScratchSpace::start();
        self.mov_instruction(MemoryAddress::direct(scratch_start), source_pointer);
        self.mov_instruction(MemoryAddress::direct(scratch_start + 1), destination_pointer);
        self.mov_instruction(MemoryAddress::direct(scratch_start + 2), num_elements_variable);
        self.add_procedure_call_instruction(ProcedureId::MemCopy);
    }
}

pub(super) fn compile_mem_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    allocate_scratch_registers!(
        brillig_context,
        [source_pointer, destination_pointer, num_elements_variable]
    );

    brillig_context.codegen_mem_copy(
        source_pointer,
        destination_pointer,
        SingleAddrVariable::new_usize(num_elements_variable),
    );
}
