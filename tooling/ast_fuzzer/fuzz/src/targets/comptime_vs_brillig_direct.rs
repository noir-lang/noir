//! Compare the execution of random ASTs between the comptime execution
//! (after converting the AST to Noir and running it through the comptime
//! interpreter) vs when everything is forced to be Brillig.
//! We choose Brillig here because it mostly matches comptime feature set
//! (e.g. `loop`, `while`, `break` and `continue` are possible)
//! This variant accesses the interpreter directly instead of going
//! through nargo, which speeds up execution but also currently
//! has some issues (inability to use prints among others).
use crate::{compare_results_comptime, create_ssa_or_die, default_ssa_options};
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::CompareComptime;
use noir_ast_fuzzer::compare::CompareOptions;
use noir_ast_fuzzer::rewrite::change_all_functions_into_unconstrained;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let config = Config {
        // Avoid break/continue
        avoid_loop_control: true,
        // Has to only use expressions valid in comptime
        comptime_friendly: true,
        // Force brillig, to generate loops that the interpreter can do but ACIR cannot.
        force_brillig: true,
        // Allow overflows half the time.
        avoid_overflow: u.arbitrary()?,
        // Slices need some parts of the stdlib that we can't just append to the source
        // the way it is currently done to support prints, because they are low level extensions.
        avoid_slices: true,
        // Use lower limits because of the interpreter, to avoid stack overflow
        max_loop_size: 5,
        max_recursive_calls: 5,
        ..Default::default()
    };

    let inputs = CompareComptime::arb(u, config, |program| {
        let options = CompareOptions::default();
        let ssa = create_ssa_or_die(
            change_all_functions_into_unconstrained(program),
            &options.onto(default_ssa_options()),
            Some("brillig"),
        );
        Ok((ssa, options))
    })?;

    let result = inputs.exec_direct(|program| {
        let options = CompareOptions::default();
        let ssa = create_ssa_or_die(
            program,
            &options.onto(default_ssa_options()),
            Some("comptime_result_wrapper"),
        );
        Ok((ssa, options))
    })?;

    compare_results_comptime(&inputs, &result)
}

#[cfg(test)]
mod tests {

    /// ```ignore
    /// NOIR_AST_FUZZER_SEED=0x6819c61400001000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz comptime_vs_brillig_direct
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 500);
    }
}
