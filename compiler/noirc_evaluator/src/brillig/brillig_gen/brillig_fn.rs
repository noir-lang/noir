use acvm::brillig_vm::brillig::RegisterOrMemory;
use iter_extended::vecmap;

use crate::{
    brillig::brillig_ir::artifact::{BrilligParameter, Label},
    ssa::ir::{
        function::{Function, FunctionId},
        types::Type,
        value::ValueId,
    },
};
use fxhash::FxHashMap as HashMap;

use super::variable_liveness::VariableLiveness;

pub(crate) struct FunctionContext {
    pub(crate) function_id: FunctionId,
    /// Map from SSA values to register or memory. Used for variables read/written across blocks.
    pub(crate) ssa_variable_to_register_or_memory: HashMap<ValueId, RegisterOrMemory>,

    pub(crate) liveness: VariableLiveness,
}

impl FunctionContext {
    pub(crate) fn new(function: &Function) -> Self {
        let id = function.id();
        Self {
            function_id: id,
            ssa_variable_to_register_or_memory: HashMap::default(),
            liveness: VariableLiveness::from_function(function),
        }
    }

    /// Creates a function label from a given SSA function id.
    pub(crate) fn function_id_to_function_label(function_id: FunctionId) -> Label {
        function_id.to_string()
    }

    fn ssa_type_to_parameter(typ: &Type) -> BrilligParameter {
        match typ {
            Type::Numeric(_) | Type::Reference => BrilligParameter::Simple,
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
