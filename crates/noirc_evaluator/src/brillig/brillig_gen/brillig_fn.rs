use std::collections::HashMap;

use acvm::acir::brillig_vm::RegisterIndex;

use crate::{
    brillig::brillig_ir::{artifact::Label, BrilligContext},
    ssa_refactor::ir::{function::FunctionId, value::ValueId},
};

pub(crate) struct FunctionContext {
    pub(crate) function_id: FunctionId,
    /// Map from SSA values to Register Indices.
    pub(crate) ssa_value_to_register: HashMap<ValueId, RegisterIndex>,
}

impl FunctionContext {
    /// Gets a `RegisterIndex` for a `ValueId`, if one already exists
    /// or creates a new `RegisterIndex` using the latest available
    /// free register.
    pub(crate) fn get_or_create_register(
        &mut self,
        brillig_context: &mut BrilligContext,
        value: ValueId,
    ) -> RegisterIndex {
        if let Some(register_index) = self.ssa_value_to_register.get(&value) {
            return *register_index;
        }

        let register = brillig_context.allocate_register();

        // Cache the `ValueId` so that if we call it again, it will
        // return the register that has just been created.
        //
        // WARNING: This assumes that a register has not been
        // modified. If a MOV instruction has overwritten the value
        // at a register, then this cache will be invalid.
        self.ssa_value_to_register.insert(value, register);

        register
    }

    /// Creates a function label from a given SSA function id.
    pub(crate) fn function_id_to_function_label(function_id: FunctionId) -> Label {
        function_id.to_string()
    }
}
