//! Module containing Brillig-gen logic specific to SSA [Function]'s.
use iter_extended::vecmap;

use crate::{
    brillig::brillig_ir::{
        artifact::BrilligParameter,
        brillig_variable::{BrilligVariable, get_bit_size_from_ssa_type},
    },
    ssa::ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        post_order::PostOrder,
        types::Type,
        value::ValueId,
    },
};
use rustc_hash::FxHashMap as HashMap;

use super::{constant_allocation::ConstantAllocation, variable_liveness::VariableLiveness};

/// Information required to compile an SSA [Function] into Brillig bytecode.
///
/// This structure is instantiated once per function and used throughout basic block code generation.
/// It can also represent a non-function context (e.g., global instantiation) to reuse block codegen logic
/// by leaving its `function_id` field unset.
#[derive(Default)]
pub(crate) struct FunctionContext {
    /// A `FunctionContext` is necessary for using a Brillig block's code gen, but sometimes
    /// such as with globals, we are not within a function and do not have a [FunctionId].
    function_id: Option<FunctionId>,
    /// Map from SSA values its allocation. Since values can be only defined once in SSA form,
    /// we insert them here on when we allocate them at their definition.
    ///
    /// Multiple variables could be assigned the same slot, because this structure accumulates
    /// historical allocations, not just the currently active ones. This is needed so that
    /// when we start processing a block, we can always look up the allocation of the variables
    /// which are live at the beginning of it, even if they were deemed dead by another block
    /// we already visited.
    ///
    /// Note that we don't use `Allocated<BrilligVariable>` here, because we create a fresh
    /// allocator for each block we process, and something that is allocated in e.g. block 1
    /// might be deallocated in block 2, so it has to be done manually.
    pub(crate) ssa_value_allocations: HashMap<ValueId, BrilligVariable>,
    /// The block ids of the function in Reverse Post Order.
    blocks: Vec<BasicBlockId>,
    /// Liveness information for each variable in the function.
    pub(crate) liveness: VariableLiveness,
    /// Information on where to allocate constants
    pub(crate) constant_allocation: ConstantAllocation,
    /// True if this function is a brillig entry point
    pub(crate) is_entry_point: bool,
}

impl FunctionContext {
    /// Creates a new function context. It will allocate parameters for all blocks and compute the liveness of every variable.
    pub(crate) fn new(function: &Function, is_entry_point: bool) -> Self {
        let id = function.id();

        let reverse_post_order = PostOrder::with_function(function).into_vec_reverse();
        let constants = ConstantAllocation::from_function(function);
        let liveness = VariableLiveness::from_function(function, &constants);

        Self {
            function_id: Some(id),
            ssa_value_allocations: HashMap::default(),
            blocks: reverse_post_order,
            liveness,
            is_entry_point,
            constant_allocation: constants,
        }
    }

    /// Get the ID of the function this context was created for.
    ///
    /// Panics if we call it when in the context created to hold
    /// data structures for global codegen only.
    pub(crate) fn function_id(&self) -> FunctionId {
        self.function_id.expect("ICE: function_id should already be set")
    }

    /// Collects the return values of a given function
    pub(crate) fn return_values(func: &Function) -> Vec<BrilligParameter> {
        func.returns()
            .unwrap_or_default()
            .iter()
            .map(|&value_id| {
                let typ = func.dfg.type_of_value(value_id);
                Self::ssa_type_to_parameter(&typ)
            })
            .collect()
    }

    /// Converts an SSA [Type] into a corresponding [BrilligParameter].
    ///
    /// This conversion defines the calling convention for Brillig functions,
    /// ensuring that SSA values are correctly mapped to memory layouts understood by the VM.
    ///
    /// # Panics
    /// Panics if called with a vector type, as a vector's memory layout cannot be inferred without runtime data.
    pub(crate) fn ssa_type_to_parameter(typ: &Type) -> BrilligParameter {
        match typ {
            Type::Numeric(_) | Type::Reference(_) => {
                BrilligParameter::SingleAddr(get_bit_size_from_ssa_type(typ))
            }
            Type::Array(item_type, size) => BrilligParameter::Array(
                vecmap(item_type.iter(), Self::ssa_type_to_parameter),
                *size,
            ),
            Type::Vector(_) => {
                panic!("ICE: Vector parameters cannot be derived from type information")
            }
            // Treat functions as field values
            Type::Function => {
                BrilligParameter::SingleAddr(get_bit_size_from_ssa_type(&Type::field()))
            }
        }
    }

    /// Iterate blocks in Post Order.
    pub(crate) fn post_order(&self) -> impl ExactSizeIterator<Item = BasicBlockId> {
        self.blocks.iter().copied().rev()
    }

    /// Iterate blocks in Reverse Post Order.
    pub(crate) fn reverse_post_order(&self) -> impl ExactSizeIterator<Item = BasicBlockId> {
        self.blocks.iter().copied()
    }
}
