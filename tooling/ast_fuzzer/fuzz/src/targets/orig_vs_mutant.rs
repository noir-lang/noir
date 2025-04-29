//! Perform random equivalence mutations on the AST and check that the
//! execution result does not change.

use crate::{compare_results, create_ssa_or_die, default_ssa_options};
use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::{CompareMutants, CompareOptions};

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let inputs = CompareMutants::arb(
        u,
        Config::default(),
        |u, program| {
            let options = CompareOptions::arbitrary(u)?;
            // TODO(#7875): Perform random mutations
            Ok((program, options))
        },
        |program, options| create_ssa_or_die(program, &options.onto(default_ssa_options()), None),
    )?;

    let result = inputs.exec()?;

    compare_results(&inputs, &result)
}
