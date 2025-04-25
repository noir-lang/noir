//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
#![no_main]

use color_eyre::eyre::{self, Context};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::ComparePasses;
use noir_ast_fuzzer_fuzz::{compare_results, create_ssa_or_die, default_ssa_options};
use noirc_evaluator::ssa;

fuzz_target!(|data: &[u8]| {
    fuzz(&mut Unstructured::new(data)).unwrap();
});

fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let options = default_ssa_options();

    // TODO(#7873): What we really want is to do the minimum number of passes on the SSA to leave it as close to the initial SSA as possible.
    // For now just test with min/max inliner aggressiveness.
    let inputs = ComparePasses::arb(
        u,
        Config::default(),
        |program| {
            create_ssa_or_die(
                program,
                &ssa::SsaEvaluatorOptions { inliner_aggressiveness: i64::MIN, ..options.clone() },
                Some("init"),
            )
        },
        |program| {
            create_ssa_or_die(
                program,
                &ssa::SsaEvaluatorOptions { inliner_aggressiveness: i64::MAX, ..options.clone() },
                Some("final"),
            )
        },
    )?;

    let result = inputs.exec().wrap_err("exec")?;

    compare_results(&inputs, &result, |inputs| [&inputs.program])
}
