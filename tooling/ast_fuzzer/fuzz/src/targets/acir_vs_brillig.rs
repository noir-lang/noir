//! Compare the execution of random ASTs between the normal execution
//! vs when everything is forced to be Brillig.
use crate::{compare_results_compiled, create_ssa_or_die, default_ssa_options};
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::{CompareOptions, ComparePipelines};
use noir_ast_fuzzer::rewrite::change_all_functions_into_unconstrained;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let config = Config {
        // We created enough bug tickets due to overflows
        avoid_overflow: true,
        // also with negative values
        avoid_negative_int_literals: true,
        // and it gets old to have to edit u128 to fit into u32 for the frontend to parse
        avoid_large_int_literals: true,
        ..Default::default()
    };

    let inputs = ComparePipelines::arb(
        u,
        config,
        |u, program| {
            let options = CompareOptions::arbitrary(u)?;
            let ssa =
                create_ssa_or_die(program, &options.onto(default_ssa_options()), Some("acir"));
            Ok((ssa, options))
        },
        |u, program| {
            let options = CompareOptions::arbitrary(u)?;
            let ssa = create_ssa_or_die(
                change_all_functions_into_unconstrained(program),
                &options.onto(default_ssa_options()),
                Some("brillig"),
            );
            Ok((ssa, options))
        },
    )?;

    let result = inputs.exec()?;

    compare_results_compiled(&inputs, &result)
}

#[cfg(test)]
mod tests {
    use crate::targets::tests::is_running_in_ci;

    /// ```ignore
    /// NOIR_ARBTEST_SEED=0x6819c61400001000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz acir_vs_brillig
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        if is_running_in_ci() {
            // TODO: Investigate function missing purity status failures.
            return;
        }
        crate::targets::tests::fuzz_with_arbtest(super::fuzz);
    }
}
