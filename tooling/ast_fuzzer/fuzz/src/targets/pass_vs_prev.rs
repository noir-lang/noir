//! Compare the execution of an SSA pass to the one preceding it
//! using the SSA interpreter.
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

        // Choose the number of passes we run on top of the initial SSA.
        let max_passes = ssa_passes.len();
        let run_passes = u.int_in_range(1..=max_passes)?;

        // Generate the initial SSA, which is considered to be step 0.
        let ssa = ssa_gen::generate_ssa(program).expect("failed to generate initial SSA");

        // Compare two consecutive passes.
        let last_pass = &ssa_passes[run_passes - 1];
        let ssa1 = if run_passes == 1 {
            ComparePass { step: 0, msg: "Initial SSA".to_string(), ssa }
        } else {
            let prev_step = run_passes - 1;
            let prev_idx = prev_step - 1;
            ComparePass {
                step: prev_step,
                msg: ssa_passes[prev_idx].msg().to_string(),
                ssa: ssa_passes[..=prev_idx].iter().fold(ssa, run_pass_or_die),
            }
        };
        let ssa2 = ComparePass {
            step: run_passes,
            msg: last_pass.msg().to_string(),
            ssa: run_pass_or_die(clone_ssa(&ssa1.ssa), last_pass),
        };

        Ok((options, ssa1, ssa2))
    })?;

    let result = inputs.exec()?;

    compare_results_interpreted(&inputs, &result)
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
