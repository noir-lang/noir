//! Verify that every SSA pass produces well-formed SSA.
//!
//! 1. generate an arbitrary AST
//! 2. lower it to the initial SSA
//! 3. run the primary pass pipeline one pass at a time
//! 4. validate the SSA after each pass
//!
//! A failure means some pass turned valid SSA into malformed SSA — e.g. a pass
//! that produces a `truncate` following a signed-integer `unchecked_sub`, which
//! [`noirc_evaluator::ssa::ssa_gen::validate_ssa`] rejects. This is a bug
//! independent of whether the interpreter or the backends happen to cope with it.
//!
//! This complements [`super::pass_vs_prev`], which checks that passes preserve
//! *behavior*; here we check that they preserve *well-formed-ness*. Running the
//! full validator after every pass is too costly to leave on for every
//! compilation, but is affordable in a fuzzer.
use crate::default_ssa_options;
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::{Config, DisplayAstAsNoir, arb_program};
use noirc_evaluator::ssa::primary_passes;
use noirc_evaluator::ssa::ssa_gen;
use noirc_frontend::monomorphization::ast::Program;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    // `avoid_index_out_of_bounds` is randomized rather than left at its default of `true`.
    // This target does not execute the program, it only checks that each pass leaves the SSA
    // well-formed, so an always-failing bounds assertion is harmless here — and an index the
    // pipeline cannot simplify is exactly the kind of shape that has broken passes before.
    let config = Config {
        avoid_overflow: u.arbitrary()?,
        avoid_index_out_of_bounds: u.ratio(3, 4)?,
        ..Config::default()
    };
    let program = arb_program(u, config)?;

    let result = validate_each_pass(&program);
    if result.is_err() {
        // Show the AST as Noir so the failure can be replicated with other `nargo` tools.
        eprintln!("---\nAST:\n{}", DisplayAstAsNoir(&program));
    }
    result
}

fn validate_each_pass(program: &Program) -> eyre::Result<()> {
    let ssa_options = default_ssa_options();
    let passes = primary_passes(&ssa_options);

    // `generate_ssa` already validates the initial SSA, so we only check each pass's output.
    // Clone so `program` survives for the AST printout on failure.
    let mut ssa = ssa_gen::generate_ssa(program.clone()).expect("failed to generate initial SSA");

    for pass in &passes {
        ssa = pass.run(ssa).map_err(|e| eyre::eyre!("pass '{}' failed to run: {e}", pass.msg()))?;
        // Prints the SSA when `NOIR_SHOW_INVALID_SSA` is set.
        ssa = ssa_gen::validate_ssa_or_err(ssa, false)
            .map_err(|e| eyre::eyre!("SSA invalid after '{}': {e}", pass.msg()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    /// ```ignore
    /// NOIR_AST_FUZZER_SEED=0x6819c61400001000 \
    /// RUST_LOG=debug \
    /// cargo test -p noir_ast_fuzzer_fuzz valid_after_pass
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        crate::targets::tests::fuzz_with_arbtest(super::fuzz, 20000);
    }
}
