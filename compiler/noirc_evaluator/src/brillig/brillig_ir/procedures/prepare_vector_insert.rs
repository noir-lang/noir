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
    /// It prepares a vector for a insert operation, leaving a hole at the index position which is returned as the write_pointer.
    pub(crate) fn call_prepare_vector_insert_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        index: SingleAddrVariable,
        write_pointer: MemoryAddress,
        item_count: usize,
    ) {
        let source_vector_pointer_arg = MemoryAddress::from(ScratchSpace::start());
        let index_arg = MemoryAddress::from(ScratchSpace::start() + 1);
        let item_count_arg = MemoryAddress::from(ScratchSpace::start() + 2);
        let new_vector_pointer_return = MemoryAddress::from(ScratchSpace::start() + 3);
        let write_pointer_return = MemoryAddress::from(ScratchSpace::start() + 4);

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.mov_instruction(index_arg, index.address);
        self.usize_const_instruction(item_count_arg, item_count.into());

        self.add_procedure_call_instruction(ProcedureId::PrepareVectorInsert);

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
        self.mov_instruction(write_pointer, write_pointer_return);
    }
}

pub(super) fn compile_prepare_vector_insert_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let source_vector_pointer_arg = MemoryAddress::from(ScratchSpace::start());
    let index_arg = MemoryAddress::from(ScratchSpace::start() + 1);
    let item_count_arg = MemoryAddress::from(ScratchSpace::start() + 2);
    let new_vector_pointer_return = MemoryAddress::from(ScratchSpace::start() + 3);
    let write_pointer_return = MemoryAddress::from(ScratchSpace::start() + 4);

    brillig_context.set_allocated_registers(vec![
        source_vector_pointer_arg,
        index_arg,
        item_count_arg,
        new_vector_pointer_return,
        write_pointer_return,
    ]);

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: new_vector_pointer_return };
    let index = SingleAddrVariable::new_usize(index_arg);

    // First we need to allocate the target vector incrementing the size by items.len()
    let source_size = brillig_context.codegen_make_vector_length(source_vector);

    let target_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.memory_op_instruction(
        source_size.address,
        item_count_arg,
        target_size.address,
        BrilligBinaryOp::Add,
    );

    brillig_context.codegen_initialize_vector(target_vector, target_size);

    // Copy the elements to the left of the index
    let source_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(source_vector);
    let target_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(target_vector);

    brillig_context.codegen_mem_copy(
        source_vector_items_pointer,
        target_vector_items_pointer,
        index,
    );

    // Compute the source pointer just at the index
    let source_pointer_at_index = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        source_vector_items_pointer,
        index_arg,
        source_pointer_at_index,
        BrilligBinaryOp::Add,
    );

    // Compute the target pointer after the inserted elements
    brillig_context.memory_op_instruction(
        target_vector_items_pointer,
        index.address,
        write_pointer_return,
        BrilligBinaryOp::Add,
    );
    let target_pointer_after_index = brillig_context.allocate_register();

    brillig_context.memory_op_instruction(
        write_pointer_return,
        item_count_arg,
        target_pointer_after_index,
        BrilligBinaryOp::Add,
    );

    // Compute the number of elements to the right of the index
    let item_count = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        source_size.address,
        index.address,
        item_count,
        BrilligBinaryOp::Sub,
    );

    // Copy the elements to the right of the index
    brillig_context.codegen_mem_copy(
        source_pointer_at_index,
        target_pointer_after_index,
        SingleAddrVariable::new_usize(item_count),
    );

    brillig_context.deallocate_register(source_pointer_at_index);
    brillig_context.deallocate_register(target_pointer_after_index);
    brillig_context.deallocate_register(item_count);
    brillig_context.deallocate_single_addr(source_size);
    brillig_context.deallocate_single_addr(target_size);
    brillig_context.deallocate_register(source_vector_items_pointer);
    brillig_context.deallocate_register(target_vector_items_pointer);
}
