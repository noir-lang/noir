use std::collections::{HashMap, HashSet};

use acvm::brillig_vm::brillig::{HeapArray, HeapVector, RegisterIndex, RegisterOrMemory};

use crate::{
    brillig::brillig_ir::{extract_register, BrilligContext},
    ssa::ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        types::{CompositeType, Type},
        value::ValueId,
    },
};

use super::brillig_fn::FunctionContext;

#[derive(Debug, Default)]
pub(crate) struct BlockVariables {
    available_variables: HashSet<ValueId>,
    available_constants: HashMap<ValueId, RegisterOrMemory>,
}

impl BlockVariables {
    pub(crate) fn new(live_in: HashSet<ValueId>, all_block_parameters: HashSet<ValueId>) -> Self {
        BlockVariables {
            available_variables: live_in
                .into_iter()
                .chain(all_block_parameters.into_iter())
                .collect(),
            ..Default::default()
        }
    }

    pub(crate) fn get_available_variables(
        &self,
        function_context: &mut FunctionContext,
    ) -> Vec<RegisterOrMemory> {
        self.available_variables
            .iter()
            .map(|value_id| {
                function_context
                    .ssa_variable_to_register_or_memory
                    .get(value_id)
                    .unwrap_or_else(|| panic!("ICE: Value not found in cache {value_id}"))
            })
            .cloned()
            .collect()
    }

    /// For a given SSA non constant value id, create and cache the a corresponding variable.
    /// This will allocate the needed registers for the variable.
    pub(crate) fn create_variable(
        &mut self,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value_id = dfg.resolve(value_id);
        let variable = allocate_value(value_id, brillig_context, dfg);

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

    /// For a given SSA value id, return the corresponding cached allocation.
    pub(crate) fn get_value(
        &mut self,
        function_context: &FunctionContext,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value_id = dfg.resolve(value_id);
        if let Some(constant) = self.available_constants.get(&value_id) {
            *constant
        } else {
            assert!(
                self.available_variables.contains(&value_id),
                "ICE: ValueId {:?} is not available",
                value_id
            );

            *function_context
                .ssa_variable_to_register_or_memory
                .get(&value_id)
                .unwrap_or_else(|| panic!("ICE: Value not found in cache {value_id}"))
        }
    }

    /// Gets or creates a constant.
    /// Constants are a special case in SSA, since they are defined and consumed every time they are used.
    /// We keep constants block-local
    pub(crate) fn get_constant(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> Option<RegisterOrMemory> {
        let value_id = dfg.resolve(value_id);
        self.available_constants.get(&value_id).cloned()
    }

    pub(crate) fn dump_constants(&mut self) {
        self.available_constants.clear();
    }

    /// Gets or creates a constant.
    /// Constants are a special case in SSA, since they are defined and consumed every time they are used.
    /// We keep constants block-local
    pub(crate) fn create_constant(
        &mut self,
        brillig_context: &mut BrilligContext,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value_id = dfg.resolve(value_id);
        let constant = allocate_value(value_id, brillig_context, dfg);
        self.available_constants.insert(value_id, constant);
        constant
    }

    /// For a given SSA value id, return the corresponding cached allocation.
    pub(crate) fn get_block_param(
        &mut self,
        function_context: &FunctionContext,
        block_id: BasicBlockId,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value_id = dfg.resolve(value_id);
        assert!(
            function_context
                .block_parameters
                .get(&block_id)
                .expect("Block not found")
                .contains(&value_id),
            "Value is not a block parameter"
        );

        *function_context
            .ssa_variable_to_register_or_memory
            .get(&value_id)
            .expect("Block param not found")
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

pub(crate) fn allocate_value(
    value_id: ValueId,
    brillig_context: &mut BrilligContext,
    dfg: &DataFlowGraph,
) -> RegisterOrMemory {
    let typ = dfg.type_of_value(value_id);

    match typ {
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
    }
}
