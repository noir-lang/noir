use iter_extended::vecmap;

use crate::{
    brillig::brillig_ir::{
        artifact::{BrilligParameter, Label},
        brillig_variable::{get_bit_size_from_ssa_type, BrilligVariable},
    },
    ssa::ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        post_order::PostOrder,
        types::Type,
        value::ValueId,
    },
};
use fxhash::FxHashMap as HashMap;

use super::variable_liveness::VariableLiveness;

pub(crate) struct FunctionContext {
    pub(crate) function_id: FunctionId,
    /// Map from SSA values its allocation. Since values can be only defined once in SSA form, we insert them here on when we allocate them at their definition.
    pub(crate) ssa_value_allocations: HashMap<ValueId, BrilligVariable>,
    /// The block ids of the function in reverse post order.
    pub(crate) blocks: Vec<BasicBlockId>,
    /// Liveness information for each variable in the function.
    pub(crate) liveness: VariableLiveness,
}

impl FunctionContext {
    /// Creates a new function context. It will allocate parameters for all blocks and compute the liveness of every variable.
    pub(crate) fn new(function: &Function) -> Self {
        let id = function.id();

        let mut reverse_post_order = Vec::new();
        reverse_post_order.extend_from_slice(PostOrder::with_function(function).as_slice());
        reverse_post_order.reverse();

        Self {
            function_id: id,
            ssa_value_allocations: HashMap::default(),
            blocks: reverse_post_order,
            liveness: VariableLiveness::from_function(function),
        }
    }

    /// Creates a function label from a given SSA function id.
    pub(crate) fn function_id_to_function_label(function_id: FunctionId) -> Label {
        function_id.to_string()
    }

    pub(crate) fn ssa_type_to_parameter(typ: &Type) -> BrilligParameter {
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
            Type::Slice(_) => {
                panic!("ICE: Slice parameters cannot be derived from type information")
            }
            _ => unimplemented!("Unsupported function parameter/return type {typ:?}"),
        }
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
