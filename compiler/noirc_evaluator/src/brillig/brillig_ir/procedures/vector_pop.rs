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
    /// Pops items from the vector, returning the new vector and the pointer to the popped items in read_pointer.
    pub(crate) fn call_vector_pop_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        read_pointer: MemoryAddress,
        item_pop_count: usize,
        back: bool,
    ) {
        let source_vector_pointer_arg = MemoryAddress::from(ScratchSpace::start());
        let item_pop_count_arg = MemoryAddress::from(ScratchSpace::start() + 1);
        let new_vector_pointer_return = MemoryAddress::from(ScratchSpace::start() + 2);
        let read_pointer_return = MemoryAddress::from(ScratchSpace::start() + 3);

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_pop_count_arg, item_pop_count.into());

        self.add_procedure_call_instruction(ProcedureId::VectorPop(back));

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
        self.mov_instruction(read_pointer, read_pointer_return);
    }
}

pub(super) fn compile_vector_pop_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    pop_back: bool,
) {
    let source_vector_pointer_arg = MemoryAddress::from(ScratchSpace::start());
    let item_pop_count_arg = MemoryAddress::from(ScratchSpace::start() + 1);
    let new_vector_pointer_return = MemoryAddress::from(ScratchSpace::start() + 2);
    let read_pointer_return = MemoryAddress::from(ScratchSpace::start() + 3);

    brillig_context.set_allocated_registers(vec![
        source_vector_pointer_arg,
        item_pop_count_arg,
        new_vector_pointer_return,
        read_pointer_return,
    ]);

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: new_vector_pointer_return };

    // First we need to allocate the target vector decrementing the size by removed_items.len()
    let source_size = brillig_context.codegen_make_vector_length(source_vector);

    let target_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.memory_op_instruction(
        source_size.address,
        item_pop_count_arg,
        target_size.address,
        BrilligBinaryOp::Sub,
    );

    brillig_context.codegen_initialize_vector(target_vector, target_size);

    // Now we offset the source pointer by removed_items.len()
    let source_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(source_vector);
    let target_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(target_vector);

    if pop_back {
        // Now we copy the source vector starting at index 0 into the target vector
        brillig_context.codegen_mem_copy(
            source_vector_items_pointer,
            target_vector_items_pointer,
            target_size,
        );
        brillig_context.memory_op_instruction(
            source_vector_items_pointer,
            target_size.address,
            read_pointer_return,
            BrilligBinaryOp::Add,
        );
    } else {
        let source_copy_pointer = brillig_context.allocate_register();
        brillig_context.memory_op_instruction(
            source_vector_items_pointer,
            item_pop_count_arg,
            source_copy_pointer,
            BrilligBinaryOp::Add,
        );

        // Now we copy the source vector starting at index removed_items.len() into the target vector
        brillig_context.codegen_mem_copy(
            source_copy_pointer,
            target_vector_items_pointer,
            target_size,
        );
        brillig_context.mov_instruction(read_pointer_return, source_vector_items_pointer);

        brillig_context.deallocate_register(source_copy_pointer);
    }

    brillig_context.deallocate_single_addr(source_size);
    brillig_context.deallocate_single_addr(target_size);
    brillig_context.deallocate_register(source_vector_items_pointer);
    brillig_context.deallocate_register(target_vector_items_pointer);
}
