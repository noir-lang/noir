//! Perform random equivalence mutations on the AST and check that the
//! execution result does not change.

use crate::{compare_results, create_ssa_or_die, default_ssa_options};
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::CompareMutants;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let options = default_ssa_options();
    let inputs = CompareMutants::arb(
        u,
        Config::default(),
        |_u, program| {
            // TODO(#7875): Perform random mutations
            Ok(program.clone())
        },
        |program| create_ssa_or_die(program, &options, None),
    )?;

    let result = inputs.exec()?;

    compare_results(&inputs, &result)
}
