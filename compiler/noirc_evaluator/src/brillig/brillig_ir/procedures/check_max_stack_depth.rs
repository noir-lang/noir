use acvm::AcirField;

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    brillig_variable::SingleAddrVariable,
    debug_show::DebugToString,
    entry_point::{MAX_STACK_FRAME_SIZE, MAX_STACK_SIZE},
    registers::{RegisterAllocator, ScratchSpace},
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    pub(crate) fn call_check_max_stack_depth_procedure(&mut self) {
        self.add_procedure_call_instruction(ProcedureId::CheckMaxStackDepth);
    }
}

pub(super) fn compile_check_max_stack_depth_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let in_range = SingleAddrVariable::new(brillig_context.allocate_register(), 1);
    brillig_context.codegen_usize_op(
        ReservedRegisters::stack_pointer(),
        in_range.address,
        BrilligBinaryOp::LessThan,
        MAX_STACK_SIZE - MAX_STACK_FRAME_SIZE,
    );
    brillig_context.codegen_constrain(in_range, Some("Stack too deep".to_string()));
    brillig_context.deallocate_single_addr(in_range);
}
