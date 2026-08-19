#[cfg(debug_assertions)]
use std::cell::Cell;

use acvm::{FieldElement, acir::AcirField};

use super::{Context, types::AcirVar};

/// The predicate set by the most recently lowered `EnableSideEffectsIf` instruction.
///
/// The inner value is private to this module so ACIR lowering cannot read it without choosing one
/// of the accessors on [`Context`].
pub(super) struct SideEffectsLatch {
    predicate: AcirVar,
    #[cfg(debug_assertions)]
    current_instruction_declares_predicate: bool,
    /// Whether the instruction currently being lowered reported that it consumes a predicate,
    /// i.e. `Instruction::requires_acir_gen_predicate`. Unlike
    /// `current_instruction_declares_predicate` this has no allowances, so it is the flag the
    /// "declared but never read" check below is stated against.
    #[cfg(debug_assertions)]
    current_instruction_requires_predicate: bool,
    /// Whether the lowering of the current instruction has read the predicate, or explicitly
    /// acknowledged not needing it via [`Context::predicate_not_needed`]. Interior mutability
    /// keeps [`Context::predicate`] a `&self` method.
    #[cfg(debug_assertions)]
    current_instruction_read_predicate: Cell<bool>,
}

impl SideEffectsLatch {
    pub(super) fn new(one: AcirVar) -> Self {
        Self {
            predicate: one,
            #[cfg(debug_assertions)]
            current_instruction_declares_predicate: false,
            #[cfg(debug_assertions)]
            current_instruction_requires_predicate: false,
            #[cfg(debug_assertions)]
            current_instruction_read_predicate: Cell::new(false),
        }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    fn get(&self) -> AcirVar {
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                self.current_instruction_declares_predicate,
                "ACIR generation read the side-effects predicate while lowering an instruction \
                 that declared it does not consume one"
            );
            self.current_instruction_read_predicate.set(true);
        }
        self.predicate
    }

    fn get_unchecked(&self) -> AcirVar {
        self.predicate
    }

    fn set(&mut self, predicate: AcirVar) {
        self.predicate = predicate;
    }

    #[cfg(debug_assertions)]
    pub(super) fn begin_instruction(&mut self, declares_predicate: bool, requires_predicate: bool) {
        self.current_instruction_declares_predicate = declares_predicate;
        self.current_instruction_requires_predicate = requires_predicate;
        self.current_instruction_read_predicate.set(false);
    }

    /// Checks the instruction just lowered read the predicate if it said it consumes one.
    ///
    /// [`Self::get`] covers the opposite direction (reading a predicate the instruction did not
    /// declare); together they keep `Instruction::requires_acir_gen_predicate` and the lowerings
    /// in step. An instruction reporting `true` while never consulting the predicate is not
    /// unsound by itself, but it is a stale over-approximation which silently blocks
    /// optimizations, and it hides a lowering that lost a guard it used to have.
    #[cfg(debug_assertions)]
    pub(super) fn end_instruction(&self) {
        debug_assert!(
            !self.current_instruction_requires_predicate
                || self.current_instruction_read_predicate.get(),
            "ACIR generation lowered an instruction which declared it consumes a predicate \
             without reading one. Either the lowering lost its predication, or \
             `requires_acir_gen_predicate` is now an over-approximation for it. If the \
             lowering path deliberately needs no predicate, say so with \
             `Context::predicate_not_needed`."
        );
    }
}

/// Why an ACIR lowering deliberately does not use a predicate.
pub(super) enum Unpredicated {
    /// The instruction is infallible and branch-independent.
    CannotFail,
    /// An unconstrained entry point is always enabled.
    UnconstrainedEntryPoint,
}

/// Why a lowering which declares it consumes a predicate needs none on this path.
///
/// Each variant is a case where the operands the predicate would gate are known at compile time,
/// so the emitted ACIR is the same whether side effects are enabled or not.
pub(super) enum PredicateNotNeeded {
    /// A constant, in-bounds index resolved the array operation at compile time.
    ConstantIndexResolvedAtCompileTime,
    /// A constant index flattened to a fixed offset, so there is no index to gate.
    ConstantFlattenedOffset,
    /// The vector's length is a compile-time constant, so the write position is exact.
    ConstantVectorLength,
    /// The index is statically known to be in bounds, so it needs no gating.
    StaticallySafeIndex,
}

/// The only safe reason to read a predicate which may belong to an earlier instruction.
pub(super) enum StaleReadIsSafe {
    /// The value is only used to detect `Const(1)` and skip otherwise unnecessary gating.
    OnlyToSkipGating,
}

impl Context<'_> {
    /// Returns the predicate of the instruction currently being lowered.
    #[cfg_attr(debug_assertions, track_caller)]
    pub(super) fn predicate(&self) -> AcirVar {
        self.side_effects.get()
    }

    /// Returns a constant one for lowering which deliberately does not use a predicate.
    pub(super) fn no_predicate(&mut self, _why: Unpredicated) -> AcirVar {
        self.acir_context.add_constant(FieldElement::one())
    }

    /// Records that this lowering path deliberately does not read the predicate, even though the
    /// instruction reports `requires_acir_gen_predicate == true`. Suppresses the
    /// "declared but never read" check in [`SideEffectsLatch::end_instruction`] for this
    /// instruction.
    pub(super) fn predicate_not_needed(&self, _why: PredicateNotNeeded) {
        #[cfg(debug_assertions)]
        self.side_effects.current_instruction_read_predicate.set(true);
    }

    /// Returns the latch when a possibly stale value is only used in a fail-safe way.
    pub(super) fn out_of_scope_predicate(&self, _why: StaleReadIsSafe) -> AcirVar {
        self.side_effects.get_unchecked()
    }

    pub(super) fn set_predicate(&mut self, predicate: AcirVar) {
        self.side_effects.set(predicate);
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::{
        acir::{SharedContext, acir_context::BrilligStdLib},
        brillig::{Brillig, BrilligOptions},
    };

    #[test]
    #[should_panic(expected = "declared it does not consume one")]
    fn predicate_rejects_an_out_of_scope_read() {
        let mut shared_context = SharedContext::default();
        let brillig = Brillig::default();
        let brillig_options = BrilligOptions::default();
        let context =
            Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

        let _ = context.predicate();
    }

    #[test]
    #[should_panic(expected = "without reading one")]
    fn end_instruction_rejects_a_declared_predicate_which_is_never_read() {
        let mut shared_context = SharedContext::default();
        let brillig = Brillig::default();
        let brillig_options = BrilligOptions::default();
        let mut context =
            Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

        context.side_effects.begin_instruction(true, true);
        context.side_effects.end_instruction();
    }

    #[test]
    fn end_instruction_accepts_a_read_predicate() {
        let mut shared_context = SharedContext::default();
        let brillig = Brillig::default();
        let brillig_options = BrilligOptions::default();
        let mut context =
            Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

        context.side_effects.begin_instruction(true, true);
        let _ = context.predicate();
        context.side_effects.end_instruction();
    }

    #[test]
    fn end_instruction_accepts_an_acknowledged_skip() {
        let mut shared_context = SharedContext::default();
        let brillig = Brillig::default();
        let brillig_options = BrilligOptions::default();
        let mut context =
            Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

        context.side_effects.begin_instruction(true, true);
        context.predicate_not_needed(PredicateNotNeeded::ConstantVectorLength);
        context.side_effects.end_instruction();
    }

    #[test]
    fn end_instruction_accepts_an_instruction_which_consumes_no_predicate() {
        let mut shared_context = SharedContext::default();
        let brillig = Brillig::default();
        let brillig_options = BrilligOptions::default();
        let mut context =
            Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

        context.side_effects.begin_instruction(false, false);
        context.side_effects.end_instruction();
    }
}
