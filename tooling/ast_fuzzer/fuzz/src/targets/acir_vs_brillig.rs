//! Compare the execution of random ASTs between the normal execution
//! vs when everything is forced to be Brillig.
use crate::targets::default_config;
use crate::{compare_results_compiled, compile_into_circuit_or_die, default_ssa_options};
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::compare::{CompareOptions, ComparePipelines};
use noir_ast_fuzzer::rewrite::change_all_functions_into_unconstrained;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let config = default_config(u)?;

    let inputs = ComparePipelines::arb(
        u,
        config,
        |u, program| {
            let options = CompareOptions::arbitrary(u)?;
            let ssa = compile_into_circuit_or_die(
                program,
                &options.onto(default_ssa_options()),
                Some("acir"),
            );
            Ok((ssa, options))
        },
        |u, program| {
            let options = CompareOptions::arbitrary(u)?;
            let ssa = compile_into_circuit_or_die(
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

    /// ```ignore
    /// NOIR_AST_FUZZER_SEED=0x6819c61400001000 \
    /// cargo test -p noir_ast_fuzzer_fuzz acir_vs_brillig
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 10000);
    }
}
