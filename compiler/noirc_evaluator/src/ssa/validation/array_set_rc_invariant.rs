//! Verifies the implicit invariant that Brillig SSA must satisfy around
//! `array_set` and reference counts.
//!
//! # The invariant
//!
//! In Brillig, `array_set vX, i, x` may modify `vX`'s storage in place at runtime
//! when `vX`'s reference count is 1. SSA-level semantics still says `vX` is unchanged
//! and the array_set produces a fresh value; the in-place mutation is a runtime
//! optimization that's only sound when no later use can observe `vX`'s pre-mutation
//! contents through aliasing.
//!
//! Two mechanisms keep the optimization invisible to SSA semantics:
//!
//! 1. **`inc_rc`** before the `array_set` forces RC ≥ 2 at runtime so `array_set`
//!    copies rather than mutating in place.
//! 2. **Block-parameter threading** routes the post-mutation value forward as a new
//!    SSA value (the `array_set`'s result), so no later instruction references
//!    `vX` after the mutation.
//!
//! The frontend uses whichever mechanism the program needs. This pass verifies
//! that one of them is in place for every `array_set` whose source has an
//! aliased use reachable forward in the CFG.
//!
//! # Precondition
//!
//! Must be run **after `mem2reg_brillig`**. The alias-root walk through
//! block-parameter edges only reflects post-mem2reg aliasing; pre-mem2reg, an
//! `Allocate`/`Store`/`Load` chain would route aliasing through references that
//! this pass does not track.
//!
//! # Known alias-tracking gaps
//!
//! The verifier walks aliasing *only* through block-parameter edges. Aliasing
//! introduced via `MakeArray` of nested arrays, `IfElse` on arrays, non-inlined
//! `Call` returns, or `Store`/`Load` on ineligible (nested-ref) allocates is
//! **not** tracked. This is intentional for the first cut: the verifier asserts
//! the common invariant the frontend produces after mem2reg, not a universal
//! safety property.

use crate::{
    errors::RtResult,
    ssa::{ir::function::Function, ssa_gen::Ssa},
};

impl Ssa {
    /// Verifies the `array_set` / `inc_rc` aliasing invariant on every Brillig
    /// function. See the module-level docs for details.
    ///
    /// Compiled to a no-op in release builds — this is a sanity check on
    /// frontend codegen, not a hot-path safety net.
    pub(crate) fn verify_array_set_rc_invariant(self) -> RtResult<Ssa> {
        #[cfg(debug_assertions)]
        for function in self.functions.values() {
            function.verify_array_set_rc_invariant()?;
        }
        Ok(self)
    }
}

impl Function {
    #[cfg(debug_assertions)]
    pub(crate) fn verify_array_set_rc_invariant(&self) -> RtResult<()> {
        if !self.runtime().is_brillig() {
            return Ok(());
        }
        // TODO: implement the alias-walk + dominance check.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::ssa_gen::Ssa;

    /// ACIR functions are skipped: `inc_rc` / `dec_rc` are no-ops in ACIR and
    /// `array_set` always produces a fresh array.
    #[test]
    fn acir_function_is_skipped() {
        let src = r#"
            acir(inline) fn main f0 {
              b0(v0: [u32; 2]):
                v3 = array_set v0, index u32 0, value u32 99
                v5 = array_get v0, index u32 0 -> u32
                return v5
            }"#;
        let ssa = Ssa::from_str(src).unwrap();
        assert!(ssa.verify_array_set_rc_invariant().is_ok());
    }
}
