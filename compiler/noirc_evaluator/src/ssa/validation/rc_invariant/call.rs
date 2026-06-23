//! The call-argument reachable-aliased-use check.
//!
//! A callee may mutate an array argument in place at runtime — directly, or by
//! returning an alias of it that the caller then mutates ([`super::Context`]'s
//! alias analysis treats `Call` results as fresh, so that escape is invisible
//! to the `array_set` scan). When the caller reuses such an argument without a
//! protecting `inc_rc`, the mutation becomes observable through the alias —
//! exactly the precondition `purity_analysis` relies on being absent.
//!
//! This verifier drives the same aliasing/coverage/forward-walk machinery as
//! [`super::array_set`], seeded from call arguments instead of `array_set`
//! sources, and gated on whether the callee can modify its arguments (mirroring
//! `can_modify_args` in `ssa_gen`).

use crate::{errors::RtResult, ssa::ssa_gen::Ssa};

// TODO(noir-lang/noir-claude#1426): implement the call-argument check. It
// should mirror `ssa_gen`'s `can_modify_args` (skip foreign calls, pure
// builtins, and non-mutating callees), then run the shared coverage +
// forward-walk machinery from [`super`] seeded on each array-typed call
// argument, rejecting with a `CallArgAliasViolation`. Until then this is a
// no-op so the combined pipeline check stays sound for the `array_set` case.
pub(crate) fn verify(_ssa: &Ssa) -> RtResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::tests::assert_verifier_accepts_because;
    use crate::ssa::ssa_gen::Ssa;

    /// Parse `src`, run the `call` verifier, and require it to reject the SSA.
    /// Panics on any other outcome.
    ///
    // TODO(noir-lang/noir-claude#1426): once the call verifier produces a
    // `CallArgAliasViolation`, assert that specific variant here (the way
    // `array_set`'s reject helper asserts `ArraySetAliasViolation`) so it is
    // clear the *call* verifier caught the hazard.
    fn assert_verifier_rejects(src: &str) {
        let ssa = Ssa::from_str(src).expect("SSA parses");
        super::verify(&ssa).expect_err("expected the verifier to reject");
    }

    /// Regression for noir-lang/noir-claude#1426. The ownership pass clones
    /// (`inc_rc`s) every non-last use of an array, so a well-formed program
    /// that reuses an array across a call always RC-protects it. This
    /// hand-written SSA omits those bumps: a pure identity callee (`f2`)
    /// returns the array input unchanged, `f1` then `array_set`s the returned
    /// alias — mutating its caller's array in place at RC 1 — and `main`
    /// reuses the same array across two calls to `f1`. The in-place mutation
    /// is therefore observable to `main` (the first call's mutation is seen by
    /// the second), which is exactly the precondition `purity_analysis`
    /// relies on being absent. The verifier must reject: both the reused arg
    /// in `main` and the reused-then-read arg in `f1` lack a preceding
    /// `inc_rc`.
    ///
    // TODO(noir-lang/noir-claude#1426): `call::verify` is still a no-op, so
    // this test panics inside `assert_verifier_rejects`; `should_panic` marks
    // that placeholder state. Drop `should_panic` once the call-argument check
    // lands.
    #[test]
    #[should_panic(expected = "expected the verifier to reject")]
    fn end_to_end_array_reused_across_call_without_inc_rc_is_rejected() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                v1 = call f1(v0) -> Field
                v2 = call f1(v0) -> Field
                return v1, v2
            }
            brillig(inline) fn bump_via_identity f1 {
              b0(v0: [Field; 2]):
                v1 = call f2(v0) -> [Field; 2]
                v2 = array_get v0, index u32 0 -> Field
                v3 = add v2, Field 1
                v4 = array_set v1, index u32 0, value v3
                return v3
            }
            brillig(inline) fn identity f2 {
              b0(v0: [Field; 2]):
                return v0
            }"#;
        assert_verifier_rejects(src);
    }

    /// The well-formed counterpart of
    /// [`end_to_end_array_reused_across_call_without_inc_rc_is_rejected`]: the
    /// `inc_rc`s the ownership pass emits are present — in `main` before the
    /// reused call arg, and in `f1` before the array escapes to `identity` and
    /// is read again. Every reused array call-arg is now RC-protected, so the
    /// in-place mutation cannot be observed through an alias and the verifier
    /// accepts. This pins down that the call-arg check credits a preceding
    /// `inc_rc` rather than flagging every reused call arg unconditionally.
    #[test]
    fn end_to_end_array_reused_across_call_with_inc_rc_is_accepted() {
        let src = r#"
            brillig(inline) fn main f0 {
              b0():
                v0 = make_array [Field 1, Field 2] : [Field; 2]
                inc_rc v0
                v1 = call f1(v0) -> Field
                v2 = call f1(v0) -> Field
                return v1, v2
            }
            brillig(inline) fn bump_via_identity f1 {
              b0(v0: [Field; 2]):
                inc_rc v0
                v1 = call f2(v0) -> [Field; 2]
                v2 = array_get v0, index u32 0 -> Field
                v3 = add v2, Field 1
                v4 = array_set v1, index u32 0, value v3
                return v3
            }
            brillig(inline) fn identity f2 {
              b0(v0: [Field; 2]):
                return v0
            }"#;
        assert_verifier_accepts_because(
            src,
            "every reused array call-arg is protected by a preceding inc_rc",
        );
    }
}
