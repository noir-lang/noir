use std::collections::HashSet;

use acvm::brillig_vm::brillig::{HeapArray, HeapVector, RegisterIndex, RegisterOrMemory};

use crate::{
    brillig::brillig_ir::{extract_register, BrilligContext},
    ssa::ir::{
        dfg::DataFlowGraph,
        types::{CompositeType, Type},
        value::{Value, ValueId},
    },
};

use super::brillig_fn::FunctionContext;

pub(crate) struct BlockVariables {
    available_variables: HashSet<ValueId>,
}

impl BlockVariables {
    pub(crate) fn new(live_in: &HashSet<ValueId>, block_params: &[ValueId]) -> Self {
        BlockVariables {
            available_variables: HashSet::from_iter(
                live_in.iter().chain(block_params.iter()).cloned(),
            ),
        }
    }

    pub(crate) fn get_available_variables(
        &self,
        function_context: &mut FunctionContext,
        dfg: &DataFlowGraph,
    ) -> Vec<RegisterOrMemory> {
        self.available_variables
            .iter()
            .filter(|value_id| {
                let value = &dfg[**value_id];
                matches!(value, Value::Param { .. } | Value::Instruction { .. })
            })
            .map(|value_id| {
                function_context
                    .ssa_variable_to_register_or_memory
                    .get(value_id)
                    .unwrap_or_else(|| panic!("ICE: Value not found in cache {value_id}"))
            })
            .cloned()
            .collect()
    }

    /// For a given SSA value id, create and cache the a corresponding variable.
    /// This will allocate the needed registers for the variable.
    pub(crate) fn create_variable(
        &mut self,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value_id = dfg.resolve(value_id);
        let typ = dfg.type_of_value(value_id);

        let variable = match typ {
            Type::Numeric(_) | Type::Reference => {
                let register = brillig_context.allocate_register();
                RegisterOrMemory::RegisterIndex(register)
            }
            Type::Array(item_typ, elem_count) => {
                let pointer_register = brillig_context.allocate_register();
                let size = compute_array_length(&item_typ, elem_count);
                RegisterOrMemory::HeapArray(HeapArray { pointer: pointer_register, size })
            }
            Type::Slice(_) => {
                let pointer_register = brillig_context.allocate_register();
                let size_register = brillig_context.allocate_register();
                RegisterOrMemory::HeapVector(HeapVector {
                    pointer: pointer_register,
                    size: size_register,
                })
            }
            Type::Function => {
                unreachable!("ICE: Function values should have been removed from the SSA")
            }
        };

        // Cache the `ValueId` so that if we call get_variable, it will
        // return the registers that have just been created.
        //
        // WARNING: This assumes that a registers won't be reused for a different value.
        // If you overwrite the registers, then the cache will be invalid.

        if function_context.ssa_variable_to_register_or_memory.insert(value_id, variable).is_some()
        {
            unreachable!("ICE: ValueId {value_id:?} was already in cache");
        }

        self.available_variables.insert(value_id);

        variable
    }

    pub(crate) fn remove_variable(&mut self, value_id: &ValueId) {
        self.available_variables.remove(value_id);
    }

    /// For a given SSA value id, return the corresponding cached variable.
    pub(crate) fn get_variable(
        &mut self,
        function_context: &FunctionContext,
        value: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value = dfg.resolve(value);
        assert!(
            self.available_variables.contains(&value),
            "ICE: ValueId {value:?} is not available",
            value = value
        );

        *function_context
            .ssa_variable_to_register_or_memory
            .get(&value)
            .unwrap_or_else(|| panic!("ICE: Value not found in cache {value}"))
    }

    pub(crate) fn get_or_create_variable(
        &mut self,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext,
        value: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value = dfg.resolve(value);
        if let Some(variable) = function_context.ssa_variable_to_register_or_memory.get(&value) {
            return *variable;
        }

        self.create_variable(function_context, brillig_context, value, dfg)
    }

    /// Creates a variable that fits in a single register and returns the register.
    pub(crate) fn create_register_variable(
        &mut self,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext,
        value: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterIndex {
        let variable = self.create_variable(function_context, brillig_context, value, dfg);
        extract_register(variable)
    }
}

/// Computes the length of an array. This will match with the indexes that SSA will issue
pub(crate) fn compute_array_length(item_typ: &CompositeType, elem_count: usize) -> usize {
    item_typ.len() * elem_count
}
