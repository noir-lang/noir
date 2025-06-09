//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
use crate::{
    compare_results_compiled, create_ssa_or_die, create_ssa_with_passes_or_die, default_ssa_options,
};
use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre;
use noir_ast_fuzzer::compare::{CompareOptions, ComparePipelines};
use noir_ast_fuzzer::{
    Config, compare::CompareResult, rewrite::change_all_functions_into_unconstrained,
};
use noirc_evaluator::ssa::minimal_passes;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let passes = minimal_passes();
    let config = Config {
        // Overflows are easy to trigger.
        avoid_overflow: u.arbitrary()?,
        ..Default::default()
    };

    let inputs = ComparePipelines::arb(
        u,
        config,
        |_u, program| {
            // We want to do the minimum possible amount of SSA passes. Brillig can get away with fewer than ACIR,
            // because ACIR needs unrolling of loops for example, so we treat everything as Brillig.
            let options = CompareOptions::default();
            let ssa = create_ssa_with_passes_or_die(
                change_all_functions_into_unconstrained(program),
                &options.onto(default_ssa_options()),
                &passes,
                |_| vec![],
                Some("init"),
            );
            Ok((ssa, options))
        },
        |u, program| {
            let options = CompareOptions::arbitrary(u)?;
            let ssa =
                create_ssa_or_die(program, &options.onto(default_ssa_options()), Some("final"));
            Ok((ssa, options))
        },
    )?;

    let result = inputs.exec()?;

    if matches!(result, CompareResult::BothFailed(_, _)) {
        Ok(())
    } else {
        compare_results_compiled(&inputs, &result)
    }
}

#[cfg(test)]
mod tests {
    use crate::targets::tests::is_running_in_ci;

    /// ```ignore
    /// NOIR_ARBTEST_SEED=0x6819c61400001000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz min_vs_full
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        if is_running_in_ci() {
            // TODO: Investigate second program constraint failures.
            return;
        }
        crate::targets::tests::fuzz_with_arbtest(super::fuzz);
    }
}
