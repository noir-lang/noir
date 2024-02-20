use acvm::FieldElement;
use iter_extended::vecmap;

use crate::{
    brillig::brillig_ir::{
        artifact::{BrilligParameter, Label},
        brillig_variable::BrilligVariable,
        BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
    },
    ssa::ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        post_order::PostOrder,
        types::{NumericType, Type},
        value::ValueId,
    },
};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{brillig_block_variables::allocate_value, variable_liveness::VariableLiveness};

pub(crate) struct FunctionContext {
    pub(crate) function_id: FunctionId,
    /// Map from SSA values its allocation. Since values can be only defined once in SSA form, we insert them here on when we allocate them at their definition.
    pub(crate) ssa_value_allocations: HashMap<ValueId, BrilligVariable>,
    /// Block parameters are pre allocated at the function level.
    pub(crate) block_parameters: HashMap<BasicBlockId, Vec<ValueId>>,
    /// The block ids of the function in reverse post order.
    pub(crate) blocks: Vec<BasicBlockId>,
    /// Liveness information for each variable in the function.
    pub(crate) liveness: VariableLiveness,
}

impl FunctionContext {
    /// Creates a new function context. It will allocate parameters for all blocks and compute the liveness of every variable.
    pub(crate) fn new(function: &Function, brillig_context: &mut BrilligContext) -> Self {
        let id = function.id();

        let mut reverse_post_order = Vec::new();
        reverse_post_order.extend_from_slice(PostOrder::with_function(function).as_slice());
        reverse_post_order.reverse();

        let mut block_parameters = HashMap::default();
        let mut ssa_variable_to_register_or_memory = HashMap::default();

        for &block_id in &reverse_post_order {
            let block = &function.dfg[block_id];
            let parameters = block.parameters().to_vec();
            parameters.iter().for_each(|&value_id| {
                let variable = allocate_value(value_id, brillig_context, &function.dfg);
                ssa_variable_to_register_or_memory.insert(value_id, variable);
            });
            block_parameters.insert(block_id, parameters);
        }

        Self {
            function_id: id,
            ssa_value_allocations: ssa_variable_to_register_or_memory,
            block_parameters,
            blocks: reverse_post_order,
            liveness: VariableLiveness::from_function(function),
        }
    }

    pub(crate) fn all_block_parameters(&self) -> HashSet<ValueId> {
        self.block_parameters.values().flat_map(|parameters| parameters.iter()).cloned().collect()
    }

    /// Creates a function label from a given SSA function id.
    pub(crate) fn function_id_to_function_label(function_id: FunctionId) -> Label {
        function_id.to_string()
    }

    fn ssa_type_to_parameter(typ: &Type) -> BrilligParameter {
        match typ {
            Type::Numeric(_) | Type::Reference(_) => {
                BrilligParameter::SingleAddr(get_bit_size_from_ssa_type(typ))
            }
            Type::Array(item_type, size) => BrilligParameter::Array(
                vecmap(item_type.iter(), |item_typ| {
                    FunctionContext::ssa_type_to_parameter(item_typ)
                }),
                *size,
            ),
            Type::Slice(item_type) => {
                BrilligParameter::Slice(vecmap(item_type.iter(), |item_typ| {
                    FunctionContext::ssa_type_to_parameter(item_typ)
                }))
            }
            _ => unimplemented!("Unsupported function parameter/return type {typ:?}"),
        }
    }

    /// Collects the parameters of a given function
    pub(crate) fn parameters(func: &Function) -> Vec<BrilligParameter> {
        func.parameters()
            .iter()
            .map(|&value_id| {
                let typ = func.dfg.type_of_value(value_id);
                FunctionContext::ssa_type_to_parameter(&typ)
            })
            .collect()
    }

    /// Collects the return values of a given function
    pub(crate) fn return_values(func: &Function) -> Vec<BrilligParameter> {
        func.returns()
            .iter()
            .map(|&value_id| {
                let typ = func.dfg.type_of_value(value_id);
                FunctionContext::ssa_type_to_parameter(&typ)
            })
            .collect()
    }
}

pub(crate) fn get_bit_size_from_ssa_type(typ: &Type) -> u32 {
    match typ {
        Type::Numeric(num_type) => match num_type {
            NumericType::Signed { bit_size } | NumericType::Unsigned { bit_size } => *bit_size,
            NumericType::NativeField => FieldElement::max_num_bits(),
        },
        Type::Reference(_) => BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
        _ => unreachable!("ICE bitwise not on a non numeric type"),
    }
}
