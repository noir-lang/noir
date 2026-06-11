//! Compare the execution of an SSA at two arbitrary points in the
//! optimization pipeline using the SSA interpreter.
//!
//! The two points are chosen independently per seed and may be any
//! distance apart — including adjacent (the original behavior) and
//! all the way from the initial SSA to the fully-optimized form. If
//! their interpretation results differ, semantic preservation has been
//! broken by some pass between them; the offending pass can then be
//! pinpointed with `nargo interpret` (or the `bisect-ssa-pass` skill).
//!
//! Comparing arbitrary-distance pairs (rather than only adjacent ones)
//! roughly multiplies the per-seed odds of catching a bug whose
//! "toggling" pass is at a fixed pipeline position: adjacent-only
//! comparison catches the bug with probability `1/N` for `N` passes,
//! while arbitrary-distance comparison catches it whenever the two
//! sampled steps straddle the toggling pass.
//!
//! By using the SSA interpreter we can execute any pass in the pipeline,
//! as opposed to the Brillig runtime, which requires a minimum number
//! of passes to be carried out to work.
use crate::{compare_results_interpreted, default_ssa_options};
use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::{CompareInterpreted, CompareOptions, ComparePass};
use noirc_evaluator::ssa::ssa_gen::Ssa;
use noirc_evaluator::ssa::{SsaPass, primary_passes, ssa_gen};

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let config = Config { avoid_overflow: u.arbitrary()?, ..Config::default() };

    let inputs = CompareInterpreted::arb(u, config, |u, program| {
        let options = CompareOptions::arbitrary(u)?;
        let ssa_options = options.onto(default_ssa_options());
        let ssa_passes = primary_passes(&ssa_options);

        // Step 0 = the initial SSA (no passes run yet); step N = the SSA
        // after running the first N passes in `ssa_passes`. We pick two
        // distinct steps `step_a < step_b` from the closed interval
        // `[0, max_passes]` and compare them.
        let max_passes = ssa_passes.len();
        let step_a = u.int_in_range(0..=max_passes - 1)?;
        let step_b = u.int_in_range(step_a + 1..=max_passes)?;

        // Generate the initial SSA, which is considered to be step 0.
        let ssa = ssa_gen::generate_ssa(program).expect("failed to generate initial SSA");

        // Run passes up to (and not including) step_a to get the state
        // at step_a. The remaining passes [step_a..step_b) are then
        // applied on a clone of that state to get step_b.
        let ssa_at_a = ssa_passes[..step_a].iter().fold(ssa, run_pass_or_die);
        let ssa1 = ComparePass {
            step: step_a,
            msg: step_msg(&ssa_passes, step_a),
            ssa: clone_ssa(&ssa_at_a),
        };
        let ssa2 = ComparePass {
            step: step_b,
            msg: step_msg(&ssa_passes, step_b),
            ssa: ssa_passes[step_a..step_b].iter().fold(ssa_at_a, run_pass_or_die),
        };

        Ok((options, ssa1, ssa2))
    })?;

    let result = inputs.exec()?;

    compare_results_interpreted(&inputs, &result)
}

/// Human-readable label for a given pipeline step:
/// - step 0 is the initial SSA, before any pass has run;
/// - step N (1 ≤ N ≤ `passes.len()`) is the SSA after running
///   `passes[N - 1]`.
fn step_msg(passes: &[SsaPass<'_>], step: usize) -> String {
    if step == 0 { "Initial SSA".to_string() } else { passes[step - 1].msg().to_string() }
}

fn run_pass_or_die(ssa: Ssa, pass: &SsaPass) -> Ssa {
    pass.run(ssa).unwrap_or_else(|e| panic!("failed to run pass {}: {e}", pass.msg()))
}

fn clone_ssa(ssa: &Ssa) -> Ssa {
    Ssa::new(ssa.functions.values().cloned().collect(), ssa.error_selector_to_type.clone())
}

#[cfg(test)]
mod tests {
    /// ```ignore
    /// NOIR_AST_FUZZER_SEED=0x6819c61400001000 \
    /// RUST_LOG=debug \
    /// cargo test -p noir_ast_fuzzer_fuzz pass_vs_prev
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 20000);
    }
}
