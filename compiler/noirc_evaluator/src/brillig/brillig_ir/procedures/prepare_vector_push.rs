use std::vec;

use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
    BrilligBinaryOp, BrilligContext,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Prepares a vector for a push operation, allocating a larger vector and copying the source vector into the destination vector.
    /// It returns the write pointer to where to put the new items.
    pub(crate) fn call_prepare_vector_push_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        write_pointer: MemoryAddress,
        item_push_count: usize,
        back: bool,
    ) {
        let source_vector_pointer_arg = MemoryAddress::from(ScratchSpace::start());
        let item_push_count_arg = MemoryAddress::from(ScratchSpace::start() + 1);
        let new_vector_pointer_return = MemoryAddress::from(ScratchSpace::start() + 2);
        let write_pointer_return = MemoryAddress::from(ScratchSpace::start() + 3);

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_push_count_arg, item_push_count.into());

        self.add_procedure_call_instruction(ProcedureId::PrepareVectorPush(back));

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
        self.mov_instruction(write_pointer, write_pointer_return);
    }
}

pub(super) fn compile_prepare_vector_push_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    push_back: bool,
) {
    let source_vector_pointer_arg = MemoryAddress::from(ScratchSpace::start());
    let item_push_count_arg = MemoryAddress::from(ScratchSpace::start() + 1);
    let new_vector_pointer_return = MemoryAddress::from(ScratchSpace::start() + 2);
    let write_pointer_return = MemoryAddress::from(ScratchSpace::start() + 3);

    brillig_context.set_allocated_registers(vec![
        source_vector_pointer_arg,
        item_push_count_arg,
        new_vector_pointer_return,
        write_pointer_return,
    ]);

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: new_vector_pointer_return };

    // First we need to allocate the target vector incrementing the size by item_push_count_arg
    let source_size = brillig_context.codegen_make_vector_length(source_vector);

    let target_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.memory_op_instruction(
        source_size.address,
        item_push_count_arg,
        target_size.address,
        BrilligBinaryOp::Add,
    );

    brillig_context.codegen_initialize_vector(target_vector, target_size);

    // Now we copy the source vector into the target vector
    let source_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(source_vector);
    let target_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(target_vector);

    if push_back {
        brillig_context.codegen_mem_copy(
            source_vector_items_pointer,
            target_vector_items_pointer,
            source_size,
        );

        brillig_context.memory_op_instruction(
            target_vector_items_pointer,
            source_size.address,
            write_pointer_return,
            BrilligBinaryOp::Add,
        );
    } else {
        brillig_context.mov_instruction(write_pointer_return, target_vector_items_pointer);

        brillig_context.memory_op_instruction(
            target_vector_items_pointer,
            item_push_count_arg,
            target_vector_items_pointer,
            BrilligBinaryOp::Add,
        );

        brillig_context.codegen_mem_copy(
            source_vector_items_pointer,
            target_vector_items_pointer,
            source_size,
        );
    }

    brillig_context.deallocate_single_addr(source_size);
    brillig_context.deallocate_single_addr(target_size);
    brillig_context.deallocate_register(source_vector_items_pointer);
    brillig_context.deallocate_register(target_vector_items_pointer);
}
