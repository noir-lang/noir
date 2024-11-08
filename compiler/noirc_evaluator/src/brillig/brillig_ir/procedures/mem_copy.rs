use std::vec;

use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    brillig_variable::SingleAddrVariable,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
    BrilligContext,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    pub(crate) fn call_mem_copy_procedure(
        &mut self,
        source_pointer: MemoryAddress,
        destination_pointer: MemoryAddress,
        num_elements_variable: MemoryAddress,
    ) {
        self.mov_instruction(MemoryAddress::direct(ScratchSpace::start()), source_pointer);
        self.mov_instruction(MemoryAddress::direct(ScratchSpace::start() + 1), destination_pointer);
        self.mov_instruction(
            MemoryAddress::direct(ScratchSpace::start() + 2),
            num_elements_variable,
        );
        self.add_procedure_call_instruction(ProcedureId::MemCopy);
    }
}

pub(super) fn compile_mem_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let source_pointer = MemoryAddress::direct(ScratchSpace::start());
    let destination_pointer = MemoryAddress::direct(ScratchSpace::start() + 1);
    let num_elements_variable = MemoryAddress::direct(ScratchSpace::start() + 2);

    brillig_context.set_allocated_registers(vec![
        source_pointer,
        destination_pointer,
        num_elements_variable,
    ]);

    brillig_context.codegen_mem_copy(
        source_pointer,
        destination_pointer,
        SingleAddrVariable::new_usize(num_elements_variable),
    );
}
