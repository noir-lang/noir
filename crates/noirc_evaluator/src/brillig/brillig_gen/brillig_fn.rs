use std::collections::HashMap;

use acvm::acir::brillig::RegisterIndex;

use crate::{
    brillig::brillig_ir::{
        artifact::{BrilligParameter, Label},
        BrilligContext,
    },
    ssa_refactor::ir::{
        function::{Function, FunctionId},
        types::Type,
        value::ValueId,
    },
};

use super::brillig_block::compute_size_of_type;

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

    /// Collects the parameters of a given function
    pub(crate) fn parameters(func: &Function) -> Vec<BrilligParameter> {
        func.parameters()
            .iter()
            .map(|&value_id| {
                let typ = func.dfg.type_of_value(value_id);
                match typ {
                    Type::Numeric(_) | Type::Reference => BrilligParameter::Register,
                    Type::Array(..) => BrilligParameter::HeapArray(compute_size_of_type(&typ)),
                    _ => unimplemented!("Unsupported function parameter type {typ:?}"),
                }
            })
            .collect()
    }

    /// Collects the return values of a given function
    pub(crate) fn return_values(func: &Function) -> Vec<BrilligParameter> {
        func.returns()
            .iter()
            .map(|&value_id| {
                let typ = func.dfg.type_of_value(value_id);
                match typ {
                    Type::Numeric(_) | Type::Reference => BrilligParameter::Register,
                    Type::Array(..) => BrilligParameter::HeapArray(compute_size_of_type(&typ)),
                    _ => unimplemented!("Unsupported return value type {typ:?}"),
                }
            })
            .collect()
    }
}
