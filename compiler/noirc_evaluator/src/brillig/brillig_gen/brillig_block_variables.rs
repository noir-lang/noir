//! This module handles allocation, tracking, and lifetime management of variables
//! within a Brillig compiled SSA basic block.
//!
//! [`BlockVariables`] maintains a set of SSA [`ValueId`]s that currently have a register
//! allocated and usable ("available") during the compilation of a single SSA block into
//! Brillig instructions. "Available" means "has a register currently allocated" — not
//! merely "is SSA-live". A value can be SSA-live but unavailable if it has been spilled
//! to the heap spill region. Spill tracking is managed separately by
//! [`SpillManager`](super::spill_manager::SpillManager).
//!
//! [`BlockVariables`] cooperates with the [`FunctionContext`](super::brillig_fn::FunctionContext)
//! to manage the mapping from SSA values to [`BrilligVariable`]s and with the [`BrilligContext`]
//! for allocating registers.
//!
//! Variables are:
//! - Allocated when first defined in a block (if not already global or hoisted to the global space).
//! - Cached for reuse to avoid redundant register allocation.
//! - Deallocated explicitly when no longer needed (as determined by SSA liveness).
use acvm::acir::brillig::lengths::{ElementTypesLength, SemanticLength, SemiFlattenedLength};
use rustc_hash::FxHashSet as HashSet;

use crate::{
    brillig::{
        assert_u32,
        brillig_ir::{
            BrilligContext,
            brillig_variable::{BrilligVariable, get_bit_size_from_ssa_type},
            registers::{Allocated, RegisterAllocator},
        },
    },
    ssa::ir::{
        types::{CompositeType, Type},
        value::ValueId,
    },
};

use super::spill_manager::RegisterState;

/// Tracks SSA variables that have a register currently allocated and usable during
/// Brillig compilation of a block.
///
/// "Available" specifically means "has a register allocated right now". Values that are
/// SSA-live but have been spilled to the heap spill region are *not* in this set.
/// Spill tracking is the responsibility of [`SpillManager`](super::spill_manager::SpillManager).
///
/// This structure is instantiated per SSA basic block and initialized from the set of
/// live-in variables that are not spilled.
///
/// It implements:
/// - A set of active [`ValueId`]s that are allocated and usable.
/// - The interface to define new variables as needed for instructions within the block.
/// - Utilities to remove, check, and retrieve variables during Brillig codegen.
#[derive(Debug, Default)]
pub(crate) struct BlockVariables {
    available_variables: HashSet<ValueId>,
}

/// The spill manager consults the set of register-resident values through this trait;
/// "available" (has a register allocated right now) is exactly "in a register".
impl RegisterState for BlockVariables {
    fn is_in_register(&self, value_id: &ValueId) -> bool {
        self.is_allocated(value_id)
    }
}

impl BlockVariables {
    /// Creates a `BlockVariables` instance. It uses the variables that are live in to the block and the global available variables (block parameters)
    pub(crate) fn new(live_in: HashSet<ValueId>) -> Self {
        BlockVariables { available_variables: live_in }
    }

    /// Removes a coalesced variable without deallocating its register.
    ///
    /// This is used for coalesced arguments that share a register with their
    /// destination block parameter — the parameter still owns the register.
    pub(crate) fn remove_variable_without_dealloc(&mut self, value_id: &ValueId) {
        assert!(self.available_variables.remove(value_id), "ICE: Variable is not available");
    }

    /// Checks if a variable is allocated and live.
    pub(crate) fn is_allocated(&self, value_id: &ValueId) -> bool {
        self.available_variables.contains(value_id)
    }

    /// Remove from available set without deallocating register.
    /// Used when a spilled variable dies — its register was already freed during spill.
    pub(crate) fn mark_unavailable(&mut self, value_id: &ValueId) {
        assert!(self.available_variables.remove(value_id), "ICE: Variable is not available");
    }

    /// Add a value back to the available set (used after reload).
    pub(crate) fn add_available(&mut self, value_id: ValueId) {
        self.available_variables.insert(value_id);
    }
}

/// Computes the length of an array. This will match with the indexes that SSA will issue
pub(crate) fn compute_array_length(
    item_typ: &CompositeType,
    elem_count: SemanticLength,
) -> SemiFlattenedLength {
    ElementTypesLength(assert_u32(item_typ.len())) * elem_count
}

/// For a given [Type], allocates the necessary registers to hold it.
pub(crate) fn allocate_value_with_type<F, Registers: RegisterAllocator>(
    brillig_context: &BrilligContext<F, Registers>,
    typ: Type,
) -> Allocated<BrilligVariable, Registers> {
    match typ {
        Type::Numeric(_) | Type::Reference(..) | Type::Function => brillig_context
            .allocate_single_addr(get_bit_size_from_ssa_type(&typ))
            .map(BrilligVariable::SingleAddr),
        Type::Array(item_typ, elem_count) => brillig_context
            .allocate_brillig_array(compute_array_length(&item_typ, elem_count))
            .map(BrilligVariable::BrilligArray),
        Type::Vector(_) => {
            brillig_context.allocate_brillig_vector().map(BrilligVariable::BrilligVector)
        }
    }
}
