//! Compare the execution of random ASTs between the normal execution
//! vs when everything is forced to be Brillig.
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

#[cfg(test)]
mod tests {
    use arbtest::arbtest;
    use std::time::Duration;

    use crate::targets::tests::seed_from_env;

    /// `cargo fuzz` takes a long time to ramp up the complexity.
    /// This test catches crash bugs much faster.
    ///
    /// Run it with for example:
    /// ```ignore
    /// NOIR_ARBTEST_SEED=0x6819c61400001000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz acir_vs_brillig
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        let mut prop = arbtest(|u| {
            super::fuzz(u).unwrap();
            Ok(())
        })
        .budget(Duration::from_secs(10))
        .size_min(1 << 12)
        .size_max(1 << 20);

        if let Some(seed) = seed_from_env() {
            prop = prop.seed(seed);
        }

        prop.run();
    }
}
