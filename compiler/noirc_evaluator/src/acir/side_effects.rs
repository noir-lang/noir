use acvm::{FieldElement, acir::AcirField};

use super::{Context, types::AcirVar};

/// The predicate set by the most recently lowered `EnableSideEffectsIf` instruction.
///
/// The inner value is private to this module so ACIR lowering cannot read it without choosing one
/// of the accessors on [`Context`].
pub(super) struct SideEffectsLatch(AcirVar);

impl SideEffectsLatch {
    pub(super) fn new(one: AcirVar) -> Self {
        Self(one)
    }

    fn get(&self) -> AcirVar {
        self.0
    }

    fn set(&mut self, predicate: AcirVar) {
        self.0 = predicate;
    }
}

/// Why an ACIR lowering deliberately does not use a predicate.
pub(super) enum Unpredicated {
    /// The instruction is infallible and branch-independent.
    CannotFail,
    /// An unconstrained entry point is always enabled.
    UnconstrainedEntryPoint,
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
        #[cfg(debug_assertions)]
        debug_assert!(
            self.current_instruction_declares_predicate,
            "ACIR generation read the side-effects predicate while lowering an instruction that \
             declared it does not consume one"
        );
        self.side_effects.get()
    }

    /// Returns a constant one for lowering which deliberately does not use a predicate.
    pub(super) fn no_predicate(&mut self, _why: Unpredicated) -> AcirVar {
        self.acir_context.add_constant(FieldElement::one())
    }

    /// Returns the latch when a possibly stale value is only used in a fail-safe way.
    pub(super) fn out_of_scope_predicate(&self, _why: StaleReadIsSafe) -> AcirVar {
        self.side_effects.get()
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
}
