//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
//!
//! ```text
//! cargo +nightly fuzz run init_vs_final -- -runs=10000 -max_len=1048576 -len_control=0
//! ```
#![no_main]

use color_eyre::eyre;
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer::compare::ComparePasses;
use noir_ast_fuzzer::{Config, compare::CompareResult};
use noir_ast_fuzzer_fuzz::{
    compare_results, create_ssa_or_die, create_ssa_with_passes_or_die, default_ssa_options,
};
use noirc_evaluator::ssa::minimal_passes;

fuzz_target!(|data: &[u8]| {
    fuzz(&mut Unstructured::new(data)).unwrap();
});

fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let options = default_ssa_options();
    let passes = minimal_passes();

    let inputs = ComparePasses::arb(
        u,
        Config::default(),
        |mut program| {
            // We want to do the minimum possible amount of SSA passes. Brillig can get away with fewer than ACIR,
            // because ACIR needs unrolling of loops for example, so we treat everything as Brillig.
            for f in program.functions.iter_mut() {
                f.unconstrained = true;
            }
            create_ssa_with_passes_or_die(program, &options, &passes, |_| vec![], Some("init"))
        },
        |program| create_ssa_or_die(program, &options, Some("final")),
    )?;

    let result = inputs.exec()?;

    // Unfortunately the minimal pipeline can fail on assertions of instructions that get eliminated from the final pipeline,
    // so if the minimal version fails and the final succeeds, it is most likely because of some overflow in a variable that
    // was ultimately unused. Therefore we only compare results if both succeeded, or if only the final failed.
    if matches!(result, CompareResult::BothFailed(_, _) | CompareResult::LeftFailed(_, _)) {
        Ok(())
    } else {
        compare_results(&inputs, &result)
    }
}
