//! Perform random equivalence mutations on the AST and check that the
//! execution result does not change.
#![no_main]

use color_eyre::eyre::{self, Context};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::CompareMutants;
use noir_ast_fuzzer_fuzz::{compare_results, create_ssa_or_die, default_ssa_options};

fuzz_target!(|data: &[u8]| {
    fuzz(&mut Unstructured::new(data)).unwrap();
});

fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
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

    let result = inputs.exec().wrap_err("exec")?;

    compare_results(&inputs, &result, |inputs| [&inputs.program.0, &inputs.program.1])
}
