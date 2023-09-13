use acvm::brillig_vm::brillig::{HeapArray, HeapVector, RegisterIndex, RegisterOrMemory};
use iter_extended::vecmap;

use crate::{
    brillig::brillig_ir::{
        artifact::{BrilligParameter, Label},
        BrilligContext,
    },
    ssa::ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        post_order::PostOrder,
        types::{CompositeType, Type},
        value::ValueId,
    },
};
use fxhash::FxHashMap as HashMap;

use super::variable_liveness::VariableLiveness;

pub(crate) struct FunctionContext {
    pub(crate) function_id: FunctionId,
    /// Map from SSA values to register or memory.
    pub(crate) ssa_value_to_brillig_variable: HashMap<ValueId, RegisterOrMemory>,

    pub(crate) blocks: Vec<BasicBlockId>,

    pub(crate) liveness: VariableLiveness,
}

impl FunctionContext {
    pub(crate) fn new(function: &Function) -> Self {
        let id = function.id();
        let mut reverse_post_order = Vec::new();
        reverse_post_order.extend_from_slice(PostOrder::with_function(function).as_slice());
        reverse_post_order.reverse();

        Self {
            function_id: id,
            ssa_value_to_brillig_variable: HashMap::default(),
            blocks: reverse_post_order,
            liveness: VariableLiveness::from_function(function),
        }
    }

    /// For a given SSA value id, create and cache the a corresponding variable.
    /// This will allocate the needed registers for the variable.
    pub(crate) fn create_variable(
        &mut self,
        brillig_context: &mut BrilligContext,
        value: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value = dfg.resolve(value);
        let typ = dfg.type_of_value(value);

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

        if self.ssa_value_to_brillig_variable.insert(value, variable).is_some() {
            unreachable!("ICE: ValueId {value:?} was already in cache");
        }

        variable
    }

    /// For a given SSA value id, return the corresponding cached variable.
    pub(crate) fn get_variable(&mut self, value: ValueId, dfg: &DataFlowGraph) -> RegisterOrMemory {
        let value = dfg.resolve(value);
        *self
            .ssa_value_to_brillig_variable
            .get(&value)
            .unwrap_or_else(|| panic!("ICE: Value not found in cache {value}"))
    }

    pub(crate) fn get_or_create_variable(
        &mut self,
        brillig_context: &mut BrilligContext,
        value: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterOrMemory {
        let value = dfg.resolve(value);
        if let Some(variable) = self.ssa_value_to_brillig_variable.get(&value) {
            return *variable;
        }

        self.create_variable(brillig_context, value, dfg)
    }

    /// Creates a variable that fits in a single register and returns the register.
    pub(crate) fn create_register_variable(
        &mut self,
        brillig_context: &mut BrilligContext,
        value: ValueId,
        dfg: &DataFlowGraph,
    ) -> RegisterIndex {
        let variable = self.create_variable(brillig_context, value, dfg);
        self.extract_register(variable)
    }

    pub(crate) fn extract_register(&self, variable: RegisterOrMemory) -> RegisterIndex {
        match variable {
            RegisterOrMemory::RegisterIndex(register_index) => register_index,
            _ => unreachable!("ICE: Expected register, got {variable:?}"),
        }
    }

    pub(crate) fn extract_heap_array(&self, variable: RegisterOrMemory) -> HeapArray {
        match variable {
            RegisterOrMemory::HeapArray(array) => array,
            _ => unreachable!("ICE: Expected array, got {variable:?}"),
        }
    }

    /// Collects the registers that a given variable is stored in.
    pub(crate) fn extract_registers(&self, variable: RegisterOrMemory) -> Vec<RegisterIndex> {
        match variable {
            RegisterOrMemory::RegisterIndex(register_index) => vec![register_index],
            RegisterOrMemory::HeapArray(array) => {
                vec![array.pointer]
            }
            RegisterOrMemory::HeapVector(vector) => {
                vec![vector.pointer, vector.size]
            }
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

/// Computes the length of an array. This will match with the indexes that SSA will issue
pub(crate) fn compute_array_length(item_typ: &CompositeType, elem_count: usize) -> usize {
    item_typ.len() * elem_count
}
