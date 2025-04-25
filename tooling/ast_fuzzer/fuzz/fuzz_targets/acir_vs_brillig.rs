//! Compare the execution of random ASTs between the normal execution
//! vs when everything is forced to be Brillig.
//!
//! ```text
//! cargo +nightly fuzz run acir_vs_brillig
//! ```
#![no_main]

use color_eyre::eyre;
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
        |_u, mut program| {
            // Change every function to be unconstrained.
            for f in program.functions.iter_mut() {
                f.unconstrained = true;
            }
            Ok(program)
        },
        |program| create_ssa_or_die(program, &options, None),
    )?;

    let result = inputs.exec()?;

    compare_results(&inputs, &result)
}
