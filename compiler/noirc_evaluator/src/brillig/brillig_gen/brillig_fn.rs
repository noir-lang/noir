//! Module containing Brillig-gen logic specific to SSA [Function]'s.
use iter_extended::vecmap;

use crate::{
    brillig::brillig_ir::{
        artifact::BrilligParameter, brillig_variable::get_bit_size_from_ssa_type, registers::Stack,
    },
    ssa::ir::{
        basic_block::BasicBlockId,
        function::{Function, FunctionId},
        post_order::PostOrder,
        types::Type,
    },
};
use rustc_hash::FxHashMap as HashMap;

use super::{
    allocator::GreedyAllocator, coalescing::CoalescingMap, constant_allocation::ConstantAllocation,
    spill_manager::SpillManager, variable_liveness::VariableLiveness,
};

/// Information required to compile an SSA [Function] into Brillig bytecode.
///
/// This structure is instantiated once per function and used throughout basic block code generation.
/// It can also represent a non-function context (e.g., global instantiation) to reuse block codegen logic
/// by leaving its `function_id` field unset.
#[derive(Default)]
pub(crate) struct FunctionContext {
    /// A `FunctionContext` is necessary for using a Brillig block's code gen, but sometimes
    /// such as with globals, we are not within a function and do not have a [`FunctionId`].
    function_id: Option<FunctionId>,
    /// The register allocator: the SSA-value → register cache, spill manager, and coalescing map.
    pub(crate) allocator: GreedyAllocator,
    /// The block ids of the function in Post Order.
    blocks: Vec<BasicBlockId>,
    /// Liveness information for each variable in the function.
    pub(crate) liveness: VariableLiveness,
    /// Information on where to allocate constants
    pub(crate) constant_allocation: ConstantAllocation,
}

impl FunctionContext {
    /// Creates a new function context. It will allocate parameters for all blocks and compute the liveness of every variable.
    /// Safety margin added to `max_live_count` when deciding whether a function needs
    /// spill infrastructure.
    ///
    /// Margin that account for temporary registers added by the code-gen on top
    /// of the registers corresponding to SSA values.
    /// This allows use to estimate conservatively the maximum number of live registers,
    /// by using `max_live_count` with a margin.
    /// `max_live_count` account for the SSA values, but also the additional ones
    /// required by various instructions.
    /// However some registers are not taken into account, such as parallel-move at block boundaries
    /// or on-demand constants. So `max_live_count` is a lower bound on actual Brillig register pressure.
    /// These registers are typically a few, so the margin is conservative and comfortable, so that
    /// functions close to the frame limit still get spill support.
    /// It can be tuned if it proves too aggressive or too conservative in practice.
    const SPILL_MARGIN: usize = 32;

    pub(crate) fn new(function: &Function, max_stack_frame_size: usize) -> Self {
        let id = function.id();

        let post_order = PostOrder::with_function(function).into_vec();
        let constants = ConstantAllocation::from_function(function);
        let liveness = VariableLiveness::from_function(function, &constants);

        // A single instruction's operands and scratch registers must all be resident at once;
        // no amount of spilling can lower it in a frame that cannot hold them. Enforce that floor
        // up front with a clear diagnostic, rather than letting codegen hit the reactive
        // "Stack frame too deep" panic once an allocation overflows. The floor is compared against
        // the *usable* slots — the frame minus the reserved prologue slots.
        let usable_registers = max_stack_frame_size.saturating_sub(Stack::START_OFFSET);
        assert!(
            liveness.min_live_count <= usable_registers,
            "Brillig function {id} has an instruction that needs at least {} registers, but only \
             {usable_registers} are usable in a frame of max_stack_frame_size {max_stack_frame_size}",
            liveness.min_live_count,
        );

        let needs_spill_support =
            liveness.max_live_count + Self::SPILL_MARGIN >= max_stack_frame_size;

        let spill_manager = if needs_spill_support { Some(SpillManager::new()) } else { None };

        // Disable coalescing when spilling is enabled.
        // Shared registers currently conflicts with the spill eviction mechanism.
        let coalescing = if spill_manager.is_some() {
            CoalescingMap::default()
        } else {
            CoalescingMap::from_function(function, &liveness)
        };

        Self {
            function_id: Some(id),
            allocator: GreedyAllocator {
                ssa_value_allocations: HashMap::default(),
                spill_manager,
                coalescing,
            },
            blocks: post_order,
            liveness,
            constant_allocation: constants,
        }
    }

    /// Whether this function has spill infrastructure enabled.
    pub(crate) fn spill_enabled(&self) -> bool {
        self.allocator.spill_manager.is_some()
    }

    /// Whether any block in this function actually spilled a value.
    pub(crate) fn did_spill(&self) -> bool {
        self.max_spill_offset() > 0
    }

    /// The number of spill slots needed (0 if no spilling occurred).
    pub(crate) fn max_spill_offset(&self) -> usize {
        self.allocator.spill_manager.as_ref().map_or(0, |sm| sm.max_spill_offset())
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

    /// Converts an SSA [Type] into a corresponding [`BrilligParameter`].
    ///
    /// This conversion defines the calling convention for Brillig functions,
    /// ensuring that SSA values are correctly mapped to memory layouts understood by the VM.
    ///
    /// # Panics
    /// Panics if called with a vector type, as a vector's memory layout cannot be inferred without runtime data.
    pub(crate) fn ssa_type_to_parameter(typ: &Type) -> BrilligParameter {
        match typ {
            Type::Numeric(_) | Type::Reference(..) | Type::Function => {
                BrilligParameter::SingleAddr(get_bit_size_from_ssa_type(typ))
            }
            Type::Array(item_type, size) => BrilligParameter::Array(
                vecmap(item_type.iter(), Self::ssa_type_to_parameter),
                *size,
            ),
            Type::Vector(_) => {
                panic!("ICE: Vector parameters cannot be derived from type information")
            }
        }
    }

    /// Iterate blocks in Post Order.
    pub(crate) fn post_order(&self) -> impl ExactSizeIterator<Item = BasicBlockId> {
        self.blocks.iter().copied()
    }

    /// Iterate blocks in Reverse Post Order.
    pub(crate) fn reverse_post_order(&self) -> impl ExactSizeIterator<Item = BasicBlockId> {
        self.blocks.iter().copied().rev()
    }
}

#[cfg(test)]
mod tests {
    use super::FunctionContext;
    use crate::ssa::ssa_gen::Ssa;

    // A signed `lt` needs max(2 inputs, 1 result) + 3 scratch = 5 usable registers at once,
    // none of which can be spilled around.
    const SIGNED_LT: &str = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = lt v0, v1
            return v2
        }
        ";

    #[test]
    #[should_panic(expected = "needs at least 5 registers, but only 4 are usable")]
    fn new_asserts_frame_fits_per_instruction_floor() {
        // Frame 6 leaves only 4 usable slots (2 are reserved for the prologue), one short of
        // the floor, so `new` must reject the layout up front instead of deferring to the
        // "Stack frame too deep" panic during codegen.
        let ssa = Ssa::from_str(SIGNED_LT).unwrap();
        FunctionContext::new(ssa.main(), 6);
    }

    #[test]
    fn new_accepts_frame_when_the_floor_fits_the_usable_slots() {
        // Frame 7 leaves 5 usable slots, exactly the floor, so the assertion passes.
        // (`new` validates only this per-instruction lower bound; because `min_live_count`
        // under-counts by the result count for scratch-bearing instructions, full codegen of
        // this `lt` actually needs one more slot — that residual is left to the codegen panic.)
        let ssa = Ssa::from_str(SIGNED_LT).unwrap();
        let ctx = FunctionContext::new(ssa.main(), 7);
        assert_eq!(ctx.liveness.min_live_count, 5);
    }
}
