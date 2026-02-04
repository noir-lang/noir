//! Compare the execution of random ASTs between the comptime execution
//! (after converting the AST to Noir and running it through the comptime
//! interpreter) vs when everything is forced to be Brillig.
//! We choose Brillig here because it mostly matches comptime feature set
//! (e.g. `loop`, `while`, `break` and `continue` are possible)
//! This variant lets nargo parse the resulting source code which is slow
//! but at the moment is more feature complete than using the interpreter
//! directly.
use crate::targets::default_config;
use crate::{compare_results_comptime, compile_into_circuit_or_die, default_ssa_options};
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
        // Match is not yet implemented in comptime.
        avoid_match: true,
        // Has to only use expressions valid in comptime
        comptime_friendly: true,
        // Force brillig, to generate loops that the interpreter can do but ACIR cannot.
        force_brillig: true,
        // Use lower limits because of the interpreter, to avoid stack overflow
        max_loop_size: 5,
        max_recursive_calls: 5,
        ..default_config(u)?
    };

    let inputs = CompareComptime::arb(u, config, |program| {
        let options = CompareOptions::default();
        let ssa = compile_into_circuit_or_die(
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
    /// NOIR_AST_FUZZER_SEED=0x6819c61400001000 \
    /// cargo test -p noir_ast_fuzzer_fuzz comptime_vs_brillig_nargo
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 1000);
    }
}
