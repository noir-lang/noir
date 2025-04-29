//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
use crate::{
    compare_results, create_ssa_or_die, create_ssa_with_passes_or_die, default_ssa_options,
};
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::compare::ComparePasses;
use noir_ast_fuzzer::{Config, change_all_functions_into_unconstrained, compare::CompareResult};
use noirc_evaluator::ssa::minimal_passes;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let options = default_ssa_options();
    let passes = minimal_passes();
    let config = Config {
        // Try to avoid using overflowing operations; see below for the reason.
        avoid_overflow: true,
        // Avoid stuff that would be difficult to copy as Noir; we want to verify these failures with nargo.
        avoid_large_int_literals: true,
        avoid_negative_int_literals: true,
        ..Default::default()
    };

    let inputs = ComparePasses::arb(
        u,
        config,
        |program| {
            // We want to do the minimum possible amount of SSA passes. Brillig can get away with fewer than ACIR,
            // because ACIR needs unrolling of loops for example, so we treat everything as Brillig.
            let program = change_all_functions_into_unconstrained(program);
            create_ssa_with_passes_or_die(program, &options, &passes, |_| vec![], Some("init"))
        },
        |program| create_ssa_or_die(program, &options, Some("final")),
    )?;

    let result = inputs.exec()?;

    // Unfortunately the minimal pipeline can fail on assertions of instructions that get eliminated from the final pipeline,
    // so if the minimal version fails and the final succeeds, it is most likely because of some overflow in a variable that
    // was ultimately unused. Therefore we only compare results if both succeeded, or if only the final failed.
    if matches!(result, CompareResult::BothFailed(_, _) | CompareResult::LeftFailed(_, _)) {
        Ok(())
    } else {
        compare_results(&inputs, &result)
    }
}

#[cfg(test)]
mod tests {
    use arbtest::arbtest;
    use std::time::Duration;

    use crate::targets::tests::{seed_from_env, should_ignore_on_ci};

    /// `cargo fuzz` takes a long time to ramp up the complexity.
    /// This test catches crash bugs much faster.
    ///
    /// Run it with for example:
    /// ```ignore
    /// export NOIR_ARBTEST_SEED=0x6819c61400001000
    /// export NOIR_AST_FUZZER_SHOW_AST=1
    /// cargo test -p noir_ast_fuzzer_fuzz init_vs_final
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        if should_ignore_on_ci() {
            return;
        }
        let mut prop = arbtest(|u| {
            super::fuzz(u).unwrap();
            Ok(())
        })
        .budget(Duration::from_secs(10))
        .size_min(1 << 12)
        .size_max(1 << 20);

        if let Some(seed) = seed_from_env() {
            prop = prop.seed(seed);
        }

        prop.run();
    }
}
