use noirc_errors::debug_info::ProcedureDebugId;
use serde::{Deserialize, Serialize};

mod array_copy;
mod array_reverse;
mod check_max_stack_depth;
mod mem_copy;
mod prepare_vector_insert;
mod prepare_vector_push;
mod revert_with_string;
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
use revert_with_string::compile_revert_with_string_procedure;
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub enum ProcedureId {
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
    RevertWithString(String),
}

impl ProcedureId {
    pub(crate) fn to_debug_id(&self) -> ProcedureDebugId {
        ProcedureDebugId(match self {
            ProcedureId::ArrayCopy => 0,
            ProcedureId::ArrayReverse => 1,
            ProcedureId::VectorCopy => 2,
            ProcedureId::MemCopy => 3,
            ProcedureId::PrepareVectorPush(true) => 4,
            ProcedureId::PrepareVectorPush(false) => 5,
            ProcedureId::VectorPopFront => 6,
            ProcedureId::VectorPopBack => 7,
            ProcedureId::PrepareVectorInsert => 8,
            ProcedureId::VectorRemove => 9,
            ProcedureId::CheckMaxStackDepth => 10,
            ProcedureId::RevertWithString(_) => 11,
        })
    }

    pub fn from_debug_id(debug_id: ProcedureDebugId) -> Self {
        let inner = debug_id.0;
        match inner {
            0 => ProcedureId::ArrayCopy,
            1 => ProcedureId::ArrayReverse,
            2 => ProcedureId::VectorCopy,
            3 => ProcedureId::MemCopy,
            4 => ProcedureId::PrepareVectorPush(true),
            5 => ProcedureId::PrepareVectorPush(false),
            6 => ProcedureId::VectorPopFront,
            7 => ProcedureId::VectorPopBack,
            8 => ProcedureId::PrepareVectorInsert,
            9 => ProcedureId::VectorRemove,
            10 => ProcedureId::CheckMaxStackDepth,
            // TODO: what to do here?
            11 => ProcedureId::RevertWithString("".to_string()),
            _ => panic!("Unsupported procedure debug ID of {inner} was supplied"),
        }
    }
}

impl std::fmt::Display for ProcedureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcedureId::ArrayCopy => write!(f, "ArrayCopy"),
            ProcedureId::ArrayReverse => write!(f, "ArrayReverse"),
            ProcedureId::VectorCopy => write!(f, "VectorCopy"),
            ProcedureId::MemCopy => write!(f, "MemCopy"),
            ProcedureId::PrepareVectorPush(flag) => write!(f, "PrepareVectorPush({flag})"),
            ProcedureId::VectorPopFront => write!(f, "VectorPopFront"),
            ProcedureId::VectorPopBack => write!(f, "VectorPopBack"),
            ProcedureId::PrepareVectorInsert => write!(f, "PrepareVectorInsert"),
            ProcedureId::VectorRemove => write!(f, "VectorRemove"),
            ProcedureId::CheckMaxStackDepth => write!(f, "CheckMaxStackDepth"),
            ProcedureId::RevertWithString(_) => write!(f, "RevertWithString"),
        }
    }
}

pub(crate) fn compile_procedure<F: AcirField + DebugToString>(
    procedure_id: ProcedureId,
) -> BrilligArtifact<F> {
    let mut brillig_context = BrilligContext::new_for_procedure(false, procedure_id.clone());
    brillig_context.enter_context(Label::procedure(procedure_id.clone()));

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
        ProcedureId::RevertWithString(revert_string) => {
            compile_revert_with_string_procedure(&mut brillig_context, revert_string);
        }
    };

    brillig_context.return_instruction();

    brillig_context.artifact()
}
