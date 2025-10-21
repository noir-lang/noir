use acvm::AcirField;

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
    brillig_variable::SingleAddrVariable,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    pub(crate) fn call_check_max_stack_depth_procedure(&mut self) {
        self.add_procedure_call_instruction(ProcedureId::CheckMaxStackDepth);
    }
}

/// The stack start should be computed earlier after allocating space for globals and the entry point.
///
/// Remember that the memory layout for entry points is as follows:
///
/// `{reserved} {scratch} {globals} {entry point (call data + return data)} {stack} {heap}`
pub(super) fn compile_check_max_stack_depth_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    stack_start: usize,
) {
    let in_range = SingleAddrVariable::new(brillig_context.allocate_register(), 1);

    let max_stack_size = brillig_context.registers().layout().max_stack_size();
    let max_frame_size = brillig_context.registers().layout().max_stack_frame_size();

    let last_possible_stack_start = stack_start + max_stack_size - max_frame_size;

    brillig_context.codegen_usize_op(
        ReservedRegisters::stack_pointer(),
        in_range.address,
        BrilligBinaryOp::LessThan,
        last_possible_stack_start,
    );
    brillig_context.codegen_constrain(in_range, Some("Stack too deep".to_string()));
    brillig_context.deallocate_single_addr(in_range);
}
