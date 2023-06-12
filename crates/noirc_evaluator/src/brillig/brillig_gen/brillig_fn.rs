use std::collections::HashMap;

use acvm::acir::brillig_vm::RegisterIndex;

use crate::ssa_refactor::ir::{function::FunctionId, value::ValueId};

pub(crate) struct FunctionContext {
    pub(crate) function_id: FunctionId,
    /// Map from SSA values to Register Indices.
    pub(crate) ssa_value_to_register: HashMap<ValueId, RegisterIndex>,
}
