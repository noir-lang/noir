use std::vec;

use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
    BrilligBinaryOp, BrilligContext,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Conditionally copies a source array/vector to a destination array/vector.
    /// If the reference count of the source array/vector is 1, then we can directly copy the pointer of the source array/vector to the destination array/vector.
    pub(crate) fn call_array_copy_procedure(
        &mut self,
        source_variable: BrilligVariable,
        destination_variable: BrilligVariable,
    ) {
        // Args
        let source_pointer = MemoryAddress::from(ScratchSpace::start());
        let size_register = MemoryAddress::from(ScratchSpace::start() + 1);
        let is_rc_one_register = MemoryAddress::from(ScratchSpace::start() + 2);

        // Returns
        let destination_pointer = MemoryAddress::from(ScratchSpace::start() + 3);

        match source_variable {
            BrilligVariable::BrilligArray(BrilligArray { pointer, rc, .. })
            | BrilligVariable::BrilligVector(BrilligVector { pointer, rc, .. }) => {
                self.mov_instruction(source_pointer, pointer);
                self.codegen_usize_op(rc, is_rc_one_register, BrilligBinaryOp::Equals, 1_usize);
            }
            _ => unreachable!("ICE: array_copy on non-array"),
        }

        match source_variable {
            BrilligVariable::BrilligArray(BrilligArray { size, .. }) => {
                self.usize_const_instruction(size_register, size.into());
            }
            BrilligVariable::BrilligVector(BrilligVector { size, .. }) => {
                self.mov_instruction(size_register, size);
            }
            _ => unreachable!("ICE: array_copy on non-array"),
        }

        self.add_procedure_call_instruction(ProcedureId::ArrayCopy);

        match destination_variable {
            BrilligVariable::BrilligArray(BrilligArray { pointer, rc, .. })
            | BrilligVariable::BrilligVector(BrilligVector { pointer, rc, .. }) => {
                self.mov_instruction(pointer, destination_pointer);
                self.usize_const_instruction(rc, 1_usize.into());
            }
            _ => unreachable!("ICE: array_copy on non-array"),
        }

        if let BrilligVariable::BrilligVector(BrilligVector { size, .. }) = destination_variable {
            self.mov_instruction(size, size_register);
        }
    }
}

pub(super) fn compile_array_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    // Args
    let source_pointer = MemoryAddress::from(ScratchSpace::start());
    let size_register = MemoryAddress::from(ScratchSpace::start() + 1);
    let is_rc_one_register = MemoryAddress::from(ScratchSpace::start() + 2);

    // Returns
    let destination_pointer = MemoryAddress::from(ScratchSpace::start() + 3);

    brillig_context.set_allocated_registers(vec![
        source_pointer,
        destination_pointer,
        size_register,
        is_rc_one_register,
    ]);

    brillig_context.codegen_branch(is_rc_one_register, |ctx, cond| {
        if cond {
            // Reference count is 1, we can mutate the array directly
            ctx.mov_instruction(destination_pointer, source_pointer);
        } else {
            // First issue a array copy to the destination
            ctx.codegen_allocate_array(destination_pointer, size_register);

            ctx.codegen_mem_copy(
                source_pointer,
                destination_pointer,
                SingleAddrVariable::new_usize(size_register),
            );
        }
    });
}
