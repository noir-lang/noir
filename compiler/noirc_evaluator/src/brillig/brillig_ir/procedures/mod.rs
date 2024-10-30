mod array_copy;
mod array_reverse;
mod check_max_stack_depth;
mod mem_copy;
mod prepare_vector_insert;
mod prepare_vector_push;
mod vector_copy;
mod vector_pop_back;
mod vector_pop_front;
mod vector_remove;

use array_copy::compile_array_copy_procedure;
use array_reverse::compile_array_reverse_procedure;
use check_max_stack_depth::compile_check_max_stack_depth_procedure;
use mem_copy::compile_mem_copy_procedure;
use prepare_vector_insert::compile_prepare_vector_insert_procedure;
use prepare_vector_push::compile_prepare_vector_push_procedure;
use vector_copy::compile_vector_copy_procedure;
use vector_pop_back::compile_vector_pop_back_procedure;
use vector_pop_front::compile_vector_pop_front_procedure;
use vector_remove::compile_vector_remove_procedure;

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
    VectorCopy,
    MemCopy,
    PrepareVectorPush(bool),
    VectorPopFront,
    VectorPopBack,
    PrepareVectorInsert,
    VectorRemove,
    CheckMaxStackDepth,
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
        ProcedureId::VectorCopy => compile_vector_copy_procedure(&mut brillig_context),
        ProcedureId::PrepareVectorPush(push_back) => {
            compile_prepare_vector_push_procedure(&mut brillig_context, push_back);
        }
        ProcedureId::VectorPopFront => {
            compile_vector_pop_front_procedure(&mut brillig_context);
        }
        ProcedureId::VectorPopBack => {
            compile_vector_pop_back_procedure(&mut brillig_context);
        }
        ProcedureId::PrepareVectorInsert => {
            compile_prepare_vector_insert_procedure(&mut brillig_context);
        }
        ProcedureId::VectorRemove => compile_vector_remove_procedure(&mut brillig_context),
        ProcedureId::CheckMaxStackDepth => {
            compile_check_max_stack_depth_procedure(&mut brillig_context);
        }
    };

    brillig_context.stop_instruction();

    brillig_context.artifact()
}
