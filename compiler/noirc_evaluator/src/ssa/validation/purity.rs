//! Validator for the invariant that every function has a purity status.
//!
//! [`Ssa::purity_analysis`][crate::ssa::opt::pure] records a purity for every function in the
//! program. A pass that mints a function afterwards has to record one for the new id as well:
//! [`FunctionPurities::purity_of`][crate::ssa::opt::pure::FunctionPurities::purity_of] returns
//! `None` for an unrecorded function, and every consumer reads `None` as "impure", so the new
//! function silently loses every optimization that purity unlocks.
//!
//! `Ssa::purity_analysis` already asserts this invariant on its own output, but the failure mode
//! lives in the window *after* the last purity analysis, where that assertion never runs. This
//! validator is the same invariant checked from [`validate_ssa`][crate::ssa::ssa_gen::validate_ssa],
//! so `--validate-between-passes` and the `valid_after_pass` fuzz target apply it after every pass.

use crate::ssa::ssa_gen::Ssa;

/// Panics if any function in `ssa` is missing a purity status.
///
/// Does nothing until a purity analysis has run. The SSA parser also populates
/// `Ssa::function_purities`, but only from the purity keywords written in the source, and a
/// hand-written SSA source is free to annotate one function and leave the rest bare.
pub(crate) fn verify_all_functions_have_purity(ssa: &Ssa) {
    if !ssa.function_purities.is_complete() {
        return;
    }

    let missing = ssa
        .functions
        .iter()
        .find(|(id, _)| ssa.function_purities.intrinsic_purity_of(**id).is_none());

    if let Some((id, function)) = missing {
        panic!("Function {} {id} does not have a purity status", function.name());
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::{
        Ssa,
        ssa_gen::{validate_ssa, validate_ssa_or_err},
    };

    /// A higher-order program: `wrapper` dynamically dispatches, so `defunctionalize` has to mint
    /// an `apply` function for it.
    const HIGHER_ORDER_SRC: &str = "
      brillig(inline) fn main f0 {
        b0(v0: u32):
          v3 = call f1(f2, v0) -> u32
          v5 = add v0, u32 1
          constrain v3 == v5
          v8 = call f1(f3, v0) -> u32
          v9 = add v0, u32 2
          constrain v8 == v9
          return
      }
      brillig(inline) fn wrapper f1 {
        b0(v0: function, v1: u32):
          v2 = call v0(v1) -> u32
          return v2
      }
      brillig(inline) fn increment f2 {
        b0(v0: u32):
          v2 = add v0, u32 1
          return v2
      }
      brillig(inline) fn increment_two f3 {
        b0(v0: u32):
          v2 = add v0, u32 2
          return v2
      }
    ";

    /// `defunctionalize` mints its `apply` functions through `Ssa::add_fn` without recording a
    /// purity for them. It is only safe in the default pipeline because it runs before the first
    /// purity analysis; run it after one and the invariant breaks.
    #[test]
    fn function_minted_after_purity_analysis_is_rejected() {
        let ssa = Ssa::from_str(HIGHER_ORDER_SRC).unwrap();
        let ssa = ssa.purity_analysis();

        // The invariant holds on the analysis' own output.
        validate_ssa(&ssa, false);

        let ssa = ssa.defunctionalize().unwrap();

        let error = validate_ssa_or_err(ssa, false)
            .err()
            .expect("a function minted after the purity analysis should fail validation");
        assert!(
            error.to_string().contains("apply f4 does not have a purity status"),
            "unexpected validation error: {error}"
        );
    }

    /// Before any purity analysis has run no function has a purity, and that is not a violation.
    #[test]
    fn functions_without_a_purity_analysis_are_accepted() {
        let ssa = Ssa::from_str(HIGHER_ORDER_SRC).unwrap();
        let ssa = ssa.defunctionalize().unwrap();

        assert!(ssa.function_purities.is_empty());
        validate_ssa(&ssa, true);
    }
}
