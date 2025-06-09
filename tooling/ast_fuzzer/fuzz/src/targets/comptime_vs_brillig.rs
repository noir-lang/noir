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
        // We created enough bug tickets due to overflows
        // TODO(#8817): Comptime code fails to compile if there is an overflow, which causes a panic.
        avoid_overflow: true,
        // also with negative values
        avoid_negative_int_literals: true,
        // also divisions
        avoid_err_by_zero: true,
        // and it gets old to have to edit u128 to fit into u32 for the frontend to parse
        avoid_large_int_literals: true,
        // Avoid break/continue
        avoid_loop_control: true,
        // TODO(#8817): Comptime code fails to compile if there is an assertion failure, which causes a panic.
        avoid_constrain: true,
        // Has to only use expressions valid in comptime
        comptime_friendly: true,
        // Force brillig
        force_brillig: true,
        // Use lower limits because of the interpreter.
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
        crate::targets::tests::fuzz_with_arbtest(super::fuzz);
    }
}
