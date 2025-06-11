//! Compare the execution of random ASTs between the comptime execution
//! (after converting the AST to Noir and running it through the comptime
//! interpreter) vs when everything is forced to be Brillig.
//! We choose Brillig here because it mostly matches comptime feature set
//! (e.g. `loop`, `while`, `break` and `continue` are possible)
use crate::{compare_results_comptime, create_ssa_or_die, default_ssa_options};
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::CompareComptime;
use noir_ast_fuzzer::compare::CompareOptions;
use noir_ast_fuzzer::rewrite::change_all_functions_into_unconstrained;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let config = Config {
        // It's easy to overflow.
        avoid_overflow: u.arbitrary()?,
        // Avoid using large integers in for loops that the frontend would reject.
        avoid_large_int_literals: true,
        // Also avoid negative integers, because the frontend rejects them for loops.
        avoid_negative_int_literals: true,
        // Avoid break/continue
        avoid_loop_control: true,
        // Has to only use expressions valid in comptime
        comptime_friendly: true,
        // Force brillig, to generate loops that the interpreter can do but ACIR cannot.
        force_brillig: true,
        // Use lower limits because of the interpreter, to avoid stack overflow
        max_loop_size: 5,
        max_recursive_calls: 5,
        ..Default::default()
    };

    let inputs = CompareComptime::arb(u, config, |_, program| {
        let options = CompareOptions::default();
        let ssa = create_ssa_or_die(
            change_all_functions_into_unconstrained(program),
            &options.onto(default_ssa_options()),
            Some("brillig"),
        );
        Ok((ssa, options))
    })?;

    let result = inputs.exec()?;

    compare_results_comptime(&inputs, &result)
}

#[cfg(test)]
mod tests {

    /// ```ignore
    /// NOIR_ARBTEST_SEED=0x6819c61400001000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz comptime_vs_brillig
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        // TODO(#8870): Allow more tests when the bug is fixed (fails in the 128th case).
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 127);
    }
}
