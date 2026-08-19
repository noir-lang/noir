use acvm::{FieldElement, acir::AcirField};

use audit::PredicateAudit;

use super::{Context, types::AcirVar};
use crate::ssa::ir::{dfg::DataFlowGraph, instruction::Instruction};

/// The predicate set by the most recently lowered `EnableSideEffectsIf` instruction.
///
/// The inner value is private to this module so ACIR lowering cannot read it without choosing one
/// of the accessors on [`Context`], which is what lets [`PredicateAudit`] hold each instruction to
/// its [`PredicateContract`].
pub(super) struct SideEffectsLatch {
    predicate: AcirVar,
    /// Enforces the contract of the instruction currently being lowered. Holds nothing and does
    /// nothing outside of debug builds.
    audit: PredicateAudit,
}

impl SideEffectsLatch {
    pub(super) fn new(one: AcirVar) -> Self {
        Self { predicate: one, audit: PredicateAudit::new() }
    }

    #[cfg_attr(debug_assertions, track_caller)]
    fn get(&self) -> AcirVar {
        self.audit.record_read();
        self.predicate
    }

    fn get_unchecked(&self) -> AcirVar {
        self.predicate
    }

    fn set(&mut self, predicate: AcirVar) {
        self.predicate = predicate;
    }

    /// Starts holding the lowering of one instruction to `contract`.
    pub(super) fn begin_instruction(&mut self, contract: PredicateContract) {
        self.audit.begin(contract);
    }

    /// Checks the lowering which just finished honoured its contract.
    pub(super) fn end_instruction(&self) {
        self.audit.end();
    }
}

/// What ACIR generation may — and must — do with the side-effects predicate while lowering one
/// instruction.
///
/// This is `Instruction::requires_acir_gen_predicate` restated as an obligation on the lowering.
/// The SSA passes (`remove_enable_side_effects`, constant folding's dedup, LICM, DIE) use that
/// method to decide whether an instruction may be moved or deduplicated across an
/// `EnableSideEffectsIf` boundary, so a lowering which consults the predicate while the method
/// reports `false` is a silent miscompile waiting to happen, and one which reports `true` without
/// consulting it is a stale over-approximation that blocks those passes. [`PredicateAudit`] checks
/// both directions on every instruction ACIR generation lowers.
///
/// [`Self::of`] is the only place the exemptions from that mirror are written down.
#[derive(Clone, Copy)]
pub(super) enum PredicateContract {
    /// The instruction consumes the predicate: its lowering must read it, or say why this path
    /// needs none ([`Context::predicate_not_needed`]).
    Consumes,
    /// The instruction may read the predicate but is not obliged to.
    Optional,
    /// The instruction declared it consumes no predicate, so reading one is a bug.
    Forbidden,
}

impl PredicateContract {
    pub(super) fn of(instruction: &Instruction, dfg: &DataFlowGraph) -> Self {
        match instruction {
            // Writes the predicate rather than reading one, so it is held to neither direction.
            Instruction::EnableSideEffectsIf { .. } => Self::Optional,
            // Reports `false` — the constraint it emits is predicated by the flattening pass, not
            // here — but may still consult the predicate while lowering its payload.
            Instruction::Constrain(..) => Self::Optional,
            _ if instruction.requires_acir_gen_predicate(dfg) => Self::Consumes,
            _ => Self::Forbidden,
        }
    }
}

/// Why an ACIR lowering deliberately does not use a predicate.
pub(super) enum Unpredicated {
    /// The instruction is infallible and branch-independent.
    CannotFail,
    /// An unconstrained entry point is always enabled.
    UnconstrainedEntryPoint,
}

/// Why a lowering of a [`PredicateContract::Consumes`] instruction needs no predicate on this path.
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
    /// instruction is [`PredicateContract::Consumes`].
    ///
    /// Prefer the helpers which record this where the decision is actually made
    /// ([`Context::index_gating_without_fallback`], [`Context::resolve_vector_length`]) over
    /// calling this from a lowering: an acknowledgment is only as true as the branch it sits on,
    /// so it belongs next to the check that establishes it.
    pub(super) fn predicate_not_needed(&self, why: PredicateNotNeeded) {
        self.side_effects.audit.record_skip(why);
    }

    /// Returns the latch when a possibly stale value is only used in a fail-safe way.
    pub(super) fn out_of_scope_predicate(&self, _why: StaleReadIsSafe) -> AcirVar {
        self.side_effects.get_unchecked()
    }

    pub(super) fn set_predicate(&mut self, predicate: AcirVar) {
        self.side_effects.set(predicate);
    }
}

/// Keeps [`Instruction::requires_acir_gen_predicate`] and the ACIR lowerings in step, by holding
/// each instruction to its [`PredicateContract`] as it is lowered.
///
/// Debug builds record what the lowering did with the predicate and assert on any drift; release
/// builds compile the whole thing away, so the accessors on [`Context`] are the same code in both.
#[cfg(debug_assertions)]
mod audit {
    use std::cell::Cell;

    use super::{PredicateContract, PredicateNotNeeded};

    pub(super) struct PredicateAudit {
        /// The contract of the instruction currently being lowered.
        contract: PredicateContract,
        /// Whether its lowering has read the predicate, or acknowledged not needing it. Interior
        /// mutability keeps [`super::Context::predicate`] a `&self` method.
        used: Cell<bool>,
    }

    impl PredicateAudit {
        pub(super) fn new() -> Self {
            // Nothing is being lowered yet, so any read is out of scope.
            Self { contract: PredicateContract::Forbidden, used: Cell::new(false) }
        }

        pub(super) fn begin(&mut self, contract: PredicateContract) {
            self.contract = contract;
            self.used.set(false);
        }

        /// Records a read of the predicate, rejecting one the instruction declared it cannot make.
        #[track_caller]
        pub(super) fn record_read(&self) {
            assert!(
                !matches!(self.contract, PredicateContract::Forbidden),
                "ACIR generation read the side-effects predicate while lowering an instruction \
                 that declared it does not consume one"
            );
            self.used.set(true);
        }

        /// Records that this lowering path needs no predicate, which discharges the obligation of
        /// a [`PredicateContract::Consumes`] instruction just as a read does.
        ///
        /// Only an instruction which owes a read can discharge one: an acknowledgment anywhere
        /// else is dead, and dead acknowledgments are how this check rots into a rubber stamp.
        #[track_caller]
        pub(super) fn record_skip(&self, _why: PredicateNotNeeded) {
            assert!(
                matches!(self.contract, PredicateContract::Consumes),
                "ACIR generation acknowledged that no predicate is needed while lowering an \
                 instruction which owes no predicate read in the first place. Either the \
                 acknowledgment sits on the wrong branch, or `requires_acir_gen_predicate` no \
                 longer reports this instruction as consuming a predicate."
            );
            self.used.set(true);
        }

        /// Checks the instruction just lowered read the predicate if it said it consumes one.
        ///
        /// [`Self::record_read`] covers the opposite direction (reading a predicate the
        /// instruction did not declare); together they keep `requires_acir_gen_predicate` and the
        /// lowerings in step. An instruction reporting `true` while never consulting the predicate
        /// is not unsound by itself, but it is a stale over-approximation which silently blocks
        /// optimizations, and it hides a lowering that lost a guard it used to have.
        pub(super) fn end(&self) {
            assert!(
                !matches!(self.contract, PredicateContract::Consumes) || self.used.get(),
                "ACIR generation lowered an instruction which declared it consumes a predicate \
                 without reading one. Either the lowering lost its predication, or \
                 `requires_acir_gen_predicate` is now an over-approximation for it. If the \
                 lowering path deliberately needs no predicate, say so with \
                 `Context::predicate_not_needed`."
            );
        }
    }
}

#[cfg(not(debug_assertions))]
mod audit {
    use super::{PredicateContract, PredicateNotNeeded};

    /// Release builds state the contract but do not check it: the checks are debug assertions, and
    /// this holds no state so `SideEffectsLatch` stays a single `AcirVar`.
    pub(super) struct PredicateAudit;

    impl PredicateAudit {
        pub(super) fn new() -> Self {
            Self
        }

        // Takes `&mut self` to match the debug build, which mutates the recorded contract, so that
        // `SideEffectsLatch` has one set of call sites rather than one per build.
        #[allow(clippy::needless_pass_by_ref_mut)]
        #[inline(always)]
        pub(super) fn begin(&mut self, _contract: PredicateContract) {}

        #[inline(always)]
        pub(super) fn record_read(&self) {}

        #[inline(always)]
        pub(super) fn record_skip(&self, _why: PredicateNotNeeded) {}

        #[inline(always)]
        pub(super) fn end(&self) {}
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

        context.side_effects.begin_instruction(PredicateContract::Consumes);
        context.side_effects.end_instruction();
    }

    #[test]
    fn end_instruction_accepts_a_read_predicate() {
        let mut shared_context = SharedContext::default();
        let brillig = Brillig::default();
        let brillig_options = BrilligOptions::default();
        let mut context =
            Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

        context.side_effects.begin_instruction(PredicateContract::Consumes);
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

        context.side_effects.begin_instruction(PredicateContract::Consumes);
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

        context.side_effects.begin_instruction(PredicateContract::Forbidden);
        context.side_effects.end_instruction();
    }

    #[test]
    #[should_panic(expected = "owes no predicate read in the first place")]
    fn a_skip_is_rejected_on_an_instruction_which_consumes_no_predicate() {
        let mut shared_context = SharedContext::default();
        let brillig = Brillig::default();
        let brillig_options = BrilligOptions::default();
        let mut context =
            Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

        context.side_effects.begin_instruction(PredicateContract::Forbidden);
        context.predicate_not_needed(PredicateNotNeeded::StaticallySafeIndex);
    }

    #[test]
    fn an_optional_predicate_may_be_read_or_left_alone() {
        let mut shared_context = SharedContext::default();
        let brillig = Brillig::default();
        let brillig_options = BrilligOptions::default();
        let mut context =
            Context::new(&mut shared_context, &brillig, BrilligStdLib::default(), &brillig_options);

        context.side_effects.begin_instruction(PredicateContract::Optional);
        context.side_effects.end_instruction();

        context.side_effects.begin_instruction(PredicateContract::Optional);
        let _ = context.predicate();
        context.side_effects.end_instruction();
    }
}
