//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
//!
//! ```text
//! cargo +nightly fuzz run init_vs_final
//! ```
#![no_main]

use color_eyre::eyre;
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::ComparePasses;
use noir_ast_fuzzer_fuzz::{
    compare_results, create_ssa_or_die, create_ssa_with_passes_or_die, default_ssa_options,
};

fuzz_target!(|data: &[u8]| {
    fuzz(&mut Unstructured::new(data)).unwrap();
});

fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let options = default_ssa_options();

    let inputs = ComparePasses::arb(
        u,
        Config::default(),
        |mut program| {
            // We won't do any SSA passes at all, but for that to have a chance to work,
            // we have to execute everything as Brillig, because ACIR needs some minimal
            // passes such as unrolling.
            for f in program.functions.iter_mut() {
                f.unconstrained = true;
            }
            create_ssa_with_passes_or_die(program, &options, &[], |_| Vec::new(), Some("init"))
        },
        |program| create_ssa_or_die(program, &options, Some("final")),
    )?;

    let result = inputs.exec()?;

    compare_results(&inputs, &result, |inputs| [&inputs.program])
}
