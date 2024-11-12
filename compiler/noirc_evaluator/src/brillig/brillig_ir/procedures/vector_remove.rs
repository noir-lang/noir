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
    /// Removes items from the vector, returning the new vector.
    pub(crate) fn call_vector_remove_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        index: SingleAddrVariable,
        item_count: usize,
    ) {
        let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
        let index_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
        let item_count_arg = MemoryAddress::direct(ScratchSpace::start() + 2);
        let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 3);

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.mov_instruction(index_arg, index.address);
        self.usize_const_instruction(item_count_arg, item_count.into());

        self.add_procedure_call_instruction(ProcedureId::VectorRemove);

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
    }
}

pub(super) fn compile_vector_remove_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
    let index_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
    let item_count_arg = MemoryAddress::direct(ScratchSpace::start() + 2);
    let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 3);

    brillig_context.set_allocated_registers(vec![
        source_vector_pointer_arg,
        index_arg,
        item_count_arg,
        new_vector_pointer_return,
    ]);

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: new_vector_pointer_return };
    let index = SingleAddrVariable::new_usize(index_arg);

    // Reallocate if necessary
    let source_size = brillig_context.codegen_make_vector_length(source_vector);

    let target_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.memory_op_instruction(
        source_size.address,
        item_count_arg,
        target_size.address,
        BrilligBinaryOp::Sub,
    );

    let rc = brillig_context.allocate_register();
    brillig_context.load_instruction(rc, source_vector.pointer);
    let is_rc_one = brillig_context.allocate_register();
    brillig_context.codegen_usize_op(rc, is_rc_one, BrilligBinaryOp::Equals, 1_usize);

    let source_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(source_vector);

    let target_vector_items_pointer = brillig_context.allocate_register();

    brillig_context.codegen_branch(is_rc_one, |brillig_context, is_rc_one| {
        if is_rc_one {
            brillig_context.mov_instruction(target_vector.pointer, source_vector.pointer);
            brillig_context.codegen_update_vector_length(target_vector, target_size);
            brillig_context
                .codegen_vector_items_pointer(target_vector, target_vector_items_pointer);
        } else {
            brillig_context.codegen_initialize_vector(target_vector, target_size, None);

            // Copy the elements to the left of the index
            brillig_context
                .codegen_vector_items_pointer(target_vector, target_vector_items_pointer);

            brillig_context.codegen_mem_copy(
                source_vector_items_pointer,
                target_vector_items_pointer,
                index,
            );
        }
    });

    // Compute the source pointer after the removed items
    let source_pointer_after_index = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        source_vector_items_pointer,
        index.address,
        source_pointer_after_index,
        BrilligBinaryOp::Add,
    );
    brillig_context.memory_op_instruction(
        source_pointer_after_index,
        item_count_arg,
        source_pointer_after_index,
        BrilligBinaryOp::Add,
    );

    // Compute the target pointer at the index
    let target_pointer_at_index = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        target_vector_items_pointer,
        index.address,
        target_pointer_at_index,
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
    brillig_context.memory_op_instruction(
        item_count,
        item_count_arg,
        item_count,
        BrilligBinaryOp::Sub,
    );

    // Copy the elements to the right of the index
    brillig_context.codegen_mem_copy(
        source_pointer_after_index,
        target_pointer_at_index,
        SingleAddrVariable::new_usize(item_count),
    );

    brillig_context.deallocate_register(rc);
    brillig_context.deallocate_register(is_rc_one);
    brillig_context.deallocate_register(source_pointer_after_index);
    brillig_context.deallocate_register(target_pointer_at_index);
    brillig_context.deallocate_register(item_count);
    brillig_context.deallocate_single_addr(source_size);
    brillig_context.deallocate_single_addr(target_size);
    brillig_context.deallocate_register(source_vector_items_pointer);
    brillig_context.deallocate_register(target_vector_items_pointer);
}
