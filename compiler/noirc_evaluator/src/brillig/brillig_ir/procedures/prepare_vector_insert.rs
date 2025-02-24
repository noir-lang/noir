use std::vec;

use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::{prepare_vector_push::reallocate_vector_for_insertion, ProcedureId};
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
        let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
        let index_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
        let item_count_arg = MemoryAddress::direct(ScratchSpace::start() + 2);
        let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 3);
        let write_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 4);

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
    let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
    let index_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
    let item_count_arg = MemoryAddress::direct(ScratchSpace::start() + 2);
    let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 3);
    let write_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 4);

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

    let source_rc = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    let source_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    let source_capacity = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    let source_items_pointer = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.codegen_read_vector_metadata(
        source_vector,
        source_rc,
        source_size,
        source_capacity,
        source_items_pointer,
    );

    // Target size is source size + item_count
    let target_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.memory_op_instruction(
        source_size.address,
        item_count_arg,
        target_size.address,
        BrilligBinaryOp::Add,
    );

    // Reallocate the target vector if necessary
    reallocate_vector_for_insertion(
        brillig_context,
        source_vector,
        source_rc,
        source_capacity,
        target_vector,
        target_size,
    );

    let target_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(target_vector);

    let was_reused = SingleAddrVariable::new(brillig_context.allocate_register(), 1);
    brillig_context.memory_op_instruction(
        source_vector.pointer,
        target_vector.pointer,
        was_reused.address,
        BrilligBinaryOp::Equals,
    );

    brillig_context.codegen_if_not(was_reused.address, |brillig_context| {
        // Copy the elements to the left of the index
        brillig_context.codegen_mem_copy(
            source_items_pointer.address,
            target_vector_items_pointer,
            index,
        );
    });

    // Compute the source pointer just at the index
    let source_pointer_at_index = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        source_items_pointer.address,
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

    // Compute the number of elements to the right of the insertion index
    let element_count_to_the_right = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        source_size.address,
        index.address,
        element_count_to_the_right,
        BrilligBinaryOp::Sub,
    );

    // Copy the elements to the right of the index
    brillig_context.codegen_mem_copy_from_the_end(
        source_pointer_at_index,
        target_pointer_after_index,
        SingleAddrVariable::new_usize(element_count_to_the_right),
    );

    brillig_context.deallocate_single_addr(source_rc);
    brillig_context.deallocate_single_addr(source_size);
    brillig_context.deallocate_single_addr(source_capacity);
    brillig_context.deallocate_single_addr(source_items_pointer);
    brillig_context.deallocate_single_addr(target_size);
    brillig_context.deallocate_register(target_vector_items_pointer);
    brillig_context.deallocate_single_addr(was_reused);
    brillig_context.deallocate_register(source_pointer_at_index);
    brillig_context.deallocate_register(target_pointer_after_index);
    brillig_context.deallocate_register(element_count_to_the_right);
}
