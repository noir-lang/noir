use acvm::AcirField;

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Call [ProcedureId::CheckMaxStackDepth].
    ///
    /// Ensures that the stack size does not grow beyond its boundaries, where it would encroach on the heap.
    pub(crate) fn call_check_max_stack_depth_procedure(&mut self) {
        self.add_procedure_call_instruction(ProcedureId::CheckMaxStackDepth);
    }
}

/// Compile [ProcedureId::CheckMaxStackDepth].
///
/// The stack start should be computed earlier after allocating space for globals and the entry point.
///
/// Remember that the memory layout for entry points is as follows:
///
/// `{reserved} {scratch} {globals} {entry point (call data + return data)} {stack} {heap}`
pub(super) fn compile_check_max_stack_depth_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    stack_start: usize,
) {
    let in_range = brillig_context.allocate_single_addr_bool();

    let layout = brillig_context.layout();
    let max_stack_size = layout.max_stack_size();
    let max_frame_size = layout.max_stack_frame_size();

    let last_possible_stack_start = stack_start + max_stack_size - max_frame_size;

    brillig_context.codegen_usize_op(
        ReservedRegisters::stack_pointer(),
        in_range.address,
        BrilligBinaryOp::LessThan,
        last_possible_stack_start,
    );
    brillig_context.codegen_constrain(*in_range, Some("Stack too deep".to_string()));
}
