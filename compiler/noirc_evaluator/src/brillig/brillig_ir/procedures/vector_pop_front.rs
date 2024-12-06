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
    /// Pops items from the front of a vector, returning the new vector
    pub(crate) fn call_vector_pop_front_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        item_pop_count: usize,
    ) {
        let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
        let item_pop_count_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
        let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 2);

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_pop_count_arg, item_pop_count.into());

        self.add_procedure_call_instruction(ProcedureId::VectorPopFront);

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
    }
}

pub(super) fn compile_vector_pop_front_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
    let item_pop_count_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
    let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 2);

    brillig_context.set_allocated_registers(vec![
        source_vector_pointer_arg,
        item_pop_count_arg,
        new_vector_pointer_return,
    ]);

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: new_vector_pointer_return };

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

    // target_size = source_size - item_pop_count
    let target_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.memory_op_instruction(
        source_size.address,
        item_pop_count_arg,
        target_size.address,
        BrilligBinaryOp::Sub,
    );

    let is_rc_one = brillig_context.allocate_register();
    brillig_context.codegen_usize_op(
        source_rc.address,
        is_rc_one,
        BrilligBinaryOp::Equals,
        1_usize,
    );

    brillig_context.codegen_branch(is_rc_one, |brillig_context, is_rc_one| {
        if is_rc_one {
            // We reuse the source vector, moving the metadata to the right (decreasing capacity)
            brillig_context.memory_op_instruction(
                source_vector.pointer,
                item_pop_count_arg,
                target_vector.pointer,
                BrilligBinaryOp::Add,
            );
            brillig_context.memory_op_instruction(
                source_capacity.address,
                item_pop_count_arg,
                source_capacity.address,
                BrilligBinaryOp::Sub,
            );
            brillig_context.codegen_initialize_vector_metadata(
                target_vector,
                target_size,
                Some(source_capacity),
            );
        } else {
            brillig_context.codegen_initialize_vector(target_vector, target_size, None);

            let target_vector_items_pointer =
                brillig_context.codegen_make_vector_items_pointer(target_vector);

            let source_copy_pointer = brillig_context.allocate_register();
            brillig_context.memory_op_instruction(
                source_items_pointer.address,
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

            brillig_context.deallocate_register(source_copy_pointer);
            brillig_context.deallocate_register(target_vector_items_pointer);
        }
    });

    brillig_context.deallocate_register(is_rc_one);
    brillig_context.deallocate_single_addr(target_size);
    brillig_context.deallocate_single_addr(source_rc);
    brillig_context.deallocate_single_addr(source_size);
    brillig_context.deallocate_single_addr(source_capacity);
    brillig_context.deallocate_single_addr(source_items_pointer);
}
