mod array_copy;
mod array_reverse;
mod mem_copy;

use array_copy::compile_array_copy_procedure;
use array_reverse::compile_array_reverse_procedure;
use mem_copy::compile_mem_copy_procedure;

use crate::brillig::brillig_ir::AcirField;

use super::{
    artifact::{BrilligArtifact, Label},
    debug_show::DebugToString,
    BrilligContext,
};

/// Procedures are a set of complex operations that are common in the noir language.
/// Extracting them to reusable procedures allows us to reduce the size of the generated Brillig.
/// Procedures receive their arguments on scratch space to avoid stack dumping&restoring.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) enum ProcedureId {
    ArrayCopy,
    ArrayReverse,
    MemCopy,
}

pub(crate) fn compile_procedure<F: AcirField + DebugToString>(
    procedure_id: ProcedureId,
) -> BrilligArtifact<F> {
    let mut brillig_context = BrilligContext::new_for_procedure(false);
    brillig_context.enter_context(Label::procedure(procedure_id));

    match procedure_id {
        ProcedureId::MemCopy => compile_mem_copy_procedure(&mut brillig_context),
        ProcedureId::ArrayCopy => compile_array_copy_procedure(&mut brillig_context),
        ProcedureId::ArrayReverse => compile_array_reverse_procedure(&mut brillig_context),
    };

    brillig_context.stop_instruction();

    brillig_context.artifact()
}
