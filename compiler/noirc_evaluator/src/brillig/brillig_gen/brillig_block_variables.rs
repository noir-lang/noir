use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    brillig::brillig_ir::{
        brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
        BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
    },
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
    block_parameters: HashSet<ValueId>,
    available_constants: HashMap<ValueId, BrilligVariable>,
}

impl BlockVariables {
    /// Creates a BlockVariables instance. It uses the variables that are live in to the block and the global available variables (block parameters)
    pub(crate) fn new(live_in: HashSet<ValueId>, all_block_parameters: HashSet<ValueId>) -> Self {
        BlockVariables {
            available_variables: live_in.into_iter().chain(all_block_parameters.clone()).collect(),
            block_parameters: all_block_parameters,
            ..Default::default()
        }
    }

    /// Returns all non-constant variables that have not been removed at this point.
    pub(crate) fn get_available_variables(
        &self,
        function_context: &FunctionContext,
    ) -> Vec<BrilligVariable> {
        self.available_variables
            .iter()
            .map(|value_id| {
                function_context
                    .ssa_value_allocations
                    .get(value_id)
                    .unwrap_or_else(|| panic!("ICE: Value not found in cache {value_id}"))
            })
            .cloned()
            .collect()
    }

    /// For a given SSA non constant value id, define the variable and return the corresponding cached allocation.
    pub(crate) fn define_variable(
        &mut self,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        let value_id = dfg.resolve(value_id);
        let variable = allocate_value(value_id, brillig_context, dfg);

        if function_context.ssa_value_allocations.insert(value_id, variable).is_some() {
            unreachable!("ICE: ValueId {value_id:?} was already in cache");
        }

        self.available_variables.insert(value_id);

        variable
    }

    /// Defines a variable that fits in a single register and returns the allocated register.
    pub(crate) fn define_single_addr_variable(
        &mut self,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext,
        value: ValueId,
        dfg: &DataFlowGraph,
    ) -> SingleAddrVariable {
        let variable = self.define_variable(function_context, brillig_context, value, dfg);
        variable.extract_single_addr()
    }

    /// Removes a variable so it's not used anymore within this block.
    pub(crate) fn remove_variable(
        &mut self,
        value_id: &ValueId,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext,
    ) {
        assert!(self.available_variables.remove(value_id), "ICE: Variable is not available");
        // Block parameters should not be deallocated
        if !self.block_parameters.contains(value_id) {
            let variable = function_context
                .ssa_value_allocations
                .get(value_id)
                .expect("ICE: Variable allocation not found");
            variable.extract_registers().iter().for_each(|register| {
                brillig_context.deallocate_register(*register);
            });
        }
    }

    /// For a given SSA value id, return the corresponding cached allocation.
    pub(crate) fn get_allocation(
        &mut self,
        function_context: &FunctionContext,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        let value_id = dfg.resolve(value_id);
        if let Some(constant) = self.available_constants.get(&value_id) {
            *constant
        } else {
            assert!(
                self.available_variables.contains(&value_id),
                "ICE: ValueId {value_id:?} is not available"
            );

            *function_context
                .ssa_value_allocations
                .get(&value_id)
                .unwrap_or_else(|| panic!("ICE: Value not found in cache {value_id}"))
        }
    }

    /// Creates a constant. Constants are a special case in SSA, since they are "defined" every time they are used.
    /// We keep constants block-local.
    pub(crate) fn allocate_constant(
        &mut self,
        brillig_context: &mut BrilligContext,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        let value_id = dfg.resolve(value_id);
        let constant = allocate_value(value_id, brillig_context, dfg);
        self.available_constants.insert(value_id, constant);
        constant
    }

    /// Gets a constant.
    pub(crate) fn get_constant(
        &mut self,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> Option<BrilligVariable> {
        let value_id = dfg.resolve(value_id);
        self.available_constants.get(&value_id).cloned()
    }

    /// Removes the allocations of all constants. Constants will need to be reallocated and reinitialized after this.
    pub(crate) fn dump_constants(&mut self) {
        self.available_constants.clear();
    }

    /// For a given block parameter, return the allocation that was done globally to the function.
    pub(crate) fn get_block_param(
        &mut self,
        function_context: &FunctionContext,
        block_id: BasicBlockId,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        let value_id = dfg.resolve(value_id);
        assert!(
            function_context
                .block_parameters
                .get(&block_id)
                .expect("Block not found")
                .contains(&value_id),
            "Value is not a block parameter"
        );

        *function_context.ssa_value_allocations.get(&value_id).expect("Block param not found")
    }
}

/// Computes the length of an array. This will match with the indexes that SSA will issue
pub(crate) fn compute_array_length(item_typ: &CompositeType, elem_count: usize) -> usize {
    item_typ.len() * elem_count
}

/// For a given value_id, allocates the necessary registers to hold it.
pub(crate) fn allocate_value(
    value_id: ValueId,
    brillig_context: &mut BrilligContext,
    dfg: &DataFlowGraph,
) -> BrilligVariable {
    let typ = dfg.type_of_value(value_id);

    match typ {
        Type::Numeric(numeric_type) => BrilligVariable::SingleAddr(SingleAddrVariable {
            address: brillig_context.allocate_register(),
            bit_size: numeric_type.bit_size(),
        }),
        Type::Reference(_) => BrilligVariable::SingleAddr(SingleAddrVariable {
            address: brillig_context.allocate_register(),
            bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
        }),
        Type::Function => {
            // NB. function references are converted to a constant when
            // translating from SSA to Brillig (to allow for debugger
            // instrumentation to work properly)
            BrilligVariable::SingleAddr(SingleAddrVariable {
                address: brillig_context.allocate_register(),
                bit_size: 32,
            })
        }
        Type::Array(item_typ, elem_count) => {
            let pointer_register = brillig_context.allocate_register();
            let rc_register = brillig_context.allocate_register();
            let size = compute_array_length(&item_typ, elem_count);

            BrilligVariable::BrilligArray(BrilligArray {
                pointer: pointer_register,
                size,
                rc: rc_register,
            })
        }
        Type::Slice(_) => {
            let pointer_register = brillig_context.allocate_register();
            let size_register = brillig_context.allocate_register();
            let rc_register = brillig_context.allocate_register();

            BrilligVariable::BrilligVector(BrilligVector {
                pointer: pointer_register,
                size: size_register,
                rc: rc_register,
            })
        }
    }
}
