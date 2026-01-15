//! This module handles allocation, tracking, and lifetime management of variables
//! within a Brillig compiled SSA basic block.
//!
//! [BlockVariables] maintains a set of SSA [ValueId]s that are live and available
//! during the compilation of a single SSA block into Brillig instructions. It cooperates
//! with the [FunctionContext] to manage the mapping from SSA values to [BrilligVariable]s
//! and with the [BrilligContext] for allocating registers.
//!
//! Variables are:
//! - Allocated when first defined in a block (if not already global or hoisted to the global space).
//! - Cached for reuse to avoid redundant register allocation.
//! - Deallocated explicitly when no longer needed (as determined by SSA liveness).
use acvm::{
    FieldElement,
    acir::brillig::lengths::{ElementTypesLength, SemanticLength, SemiFlattenedLength},
};
use rustc_hash::FxHashSet as HashSet;

use crate::{
    brillig::{
        assert_u32,
        brillig_ir::{
            BrilligContext,
            brillig_variable::{BrilligVariable, SingleAddrVariable, get_bit_size_from_ssa_type},
            registers::{Allocated, RegisterAllocator},
        },
    },
    ssa::ir::{
        dfg::DataFlowGraph,
        types::{CompositeType, Type},
        value::ValueId,
    },
};

use super::brillig_fn::FunctionContext;

/// Tracks SSA variables that are live and usable during Brillig compilation of a block.
///
/// This structure is meant to be instantiated per SSA basic block and initialized using the
/// the set of live variables that must be available at the block's entry.
///
/// It implements:
/// - A set of active [ValueId]s that are allocated and usable.
/// - The interface to define new variables as needed for instructions within the block.
/// - Utilities to remove, check, and retrieve variables during Brillig codegen.
#[derive(Debug, Default)]
pub(crate) struct BlockVariables {
    available_variables: HashSet<ValueId>,
}

impl BlockVariables {
    /// Creates a BlockVariables instance. It uses the variables that are live in to the block and the global available variables (block parameters)
    pub(crate) fn new(live_in: HashSet<ValueId>) -> Self {
        BlockVariables { available_variables: live_in }
    }

    /// Returns the allocations for all variables that are currently available.
    ///
    /// These might have been technically removed earlier in a different block,
    /// because of the order in which we process blocks, but we expect that the
    /// `FunctionContext` will remember the earlier allocations, and return them
    /// so that we can restore them as pre-allocations in the next slot.
    pub(crate) fn get_available_variable_allocations(
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
            .copied()
            .collect()
    }

    /// For a given SSA value id, define the variable and return the corresponding cached memory allocation.
    ///
    /// The allocation will be cached in [FunctionContext::ssa_value_allocations], which is how it will be
    /// passed on to the next block as a pre-allocated register, if it's still alive at that point.
    ///
    /// The variable is added to [Self::available_variables] to show that it's live, where it stays until
    /// [Self::remove_variable] deletes it.
    pub(crate) fn define_variable<Registers: RegisterAllocator>(
        &mut self,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext<FieldElement, Registers>,
        value_id: ValueId,
        dfg: &DataFlowGraph,
    ) -> BrilligVariable {
        let allocated = allocate_value(value_id, brillig_context, dfg);

        // Allocators get replaced in each block, with all visible variables becoming pre-allocated
        // in the next block; what is allocated in one block might be deallocated in the next block,
        // in a different allocator. Because of this we can detach from the allocator, and have to
        // manage deallocation manually.
        let variable = allocated.detach();

        if function_context.ssa_value_allocations.insert(value_id, variable).is_some() {
            unreachable!("ICE: ValueId {value_id:?} was already in cache");
        }

        self.available_variables.insert(value_id);

        variable
    }

    /// Defines a variable that fits in a single register and returns the allocated register.
    pub(crate) fn define_single_addr_variable<Registers: RegisterAllocator>(
        &mut self,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext<FieldElement, Registers>,
        value: ValueId,
        dfg: &DataFlowGraph,
    ) -> SingleAddrVariable {
        let variable = self.define_variable(function_context, brillig_context, value, dfg);
        variable.extract_single_addr()
    }

    /// Removes a variable so it's not used anymore within this block.
    pub(crate) fn remove_variable<Registers: RegisterAllocator>(
        &mut self,
        value_id: &ValueId,
        function_context: &mut FunctionContext,
        brillig_context: &mut BrilligContext<FieldElement, Registers>,
    ) {
        assert!(self.available_variables.remove(value_id), "ICE: Variable is not available");

        // Do not remove the allocation, just get it so we can mark it as free in memory.
        let variable = function_context
            .ssa_value_allocations
            .get(value_id)
            .expect("ICE: Variable allocation not found");

        // Explicitly deallocate the memory; note that this might be a different register
        // than the one the variable was defined in, which is why don't rely on automation.
        let register = variable.extract_register();
        brillig_context.deallocate_register(register);
    }

    /// Checks if a variable is allocated and live.
    pub(crate) fn is_allocated(&self, value_id: &ValueId) -> bool {
        self.available_variables.contains(value_id)
    }

    /// For a given SSA value id, return the corresponding cached allocation.
    ///
    /// Panics if
    /// * the variable is not in [Self::available_variables], which means it is no longer live
    /// * the variable is not in [FunctionContext::ssa_value_allocations], which means it was never defined
    pub(crate) fn get_allocation(
        &mut self,
        function_context: &FunctionContext,
        value_id: ValueId,
    ) -> BrilligVariable {
        assert!(
            self.available_variables.contains(&value_id),
            "ICE: ValueId {value_id:?} is not available"
        );

        let variable = function_context
            .ssa_value_allocations
            .get(&value_id)
            .unwrap_or_else(|| panic!("ICE: Value not found in cache {value_id}"));

        *variable
    }
}

/// Computes the length of an array. This will match with the indexes that SSA will issue
pub(crate) fn compute_array_length(
    item_typ: &CompositeType,
    elem_count: SemanticLength,
) -> SemiFlattenedLength {
    ElementTypesLength(assert_u32(item_typ.len())) * elem_count
}

/// For a given [ValueId], allocates the necessary registers to hold it.
pub(crate) fn allocate_value<F, Registers: RegisterAllocator>(
    value_id: ValueId,
    brillig_context: &mut BrilligContext<F, Registers>,
    dfg: &DataFlowGraph,
) -> Allocated<BrilligVariable, Registers> {
    let typ = dfg.type_of_value(value_id);

    allocate_value_with_type(brillig_context, typ)
}

/// For a given [Type], allocates the necessary registers to hold it.
pub(crate) fn allocate_value_with_type<F, Registers: RegisterAllocator>(
    brillig_context: &mut BrilligContext<F, Registers>,
    typ: Type,
) -> Allocated<BrilligVariable, Registers> {
    match typ {
        Type::Numeric(_) | Type::Reference(_) | Type::Function => brillig_context
            .allocate_single_addr(get_bit_size_from_ssa_type(&typ))
            .map(BrilligVariable::SingleAddr),
        Type::Array(item_typ, elem_count) => brillig_context
            .allocate_brillig_array(compute_array_length(&item_typ, SemanticLength(elem_count)))
            .map(BrilligVariable::BrilligArray),
        Type::Vector(_) => {
            brillig_context.allocate_brillig_vector().map(BrilligVariable::BrilligVector)
        }
    }
}
