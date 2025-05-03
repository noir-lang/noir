//! Compare the execution of random ASTs between the comptime execution
//! (after converting the AST to Noir and running it through the comptime
//! interpreter) vs when everything is forced to be Brillig.
use crate::{compare_results, create_ssa_or_die, default_ssa_options};
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::change_all_functions_into_unconstrained;
use noir_ast_fuzzer::compare::CompareOptions;
use noir_ast_fuzzer::compare::ComparePasses;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let config = Config {
        // We created enough bug tickets due to overflows
        avoid_overflow: true,
        // also with negative values
        avoid_negative_int_literals: true,
        // and it gets old to have to edit u128 to fit into u32 for the frontend to parse
        avoid_large_int_literals: true,
        ..Default::default()
    };

    let inputs = ComparePasses::arb(
        u,
        config,
        |u, program| {
            let options = CompareOptions::default();
            let ssa =
                create_ssa_or_die(program, &options.onto(default_ssa_options()), Some("acir"));
            Ok((ssa, options))
        },
        |u, program| {
            let options = CompareOptions::default();
            let ssa = create_ssa_or_die(
                change_all_functions_into_unconstrained(program),
                &options.onto(default_ssa_options()),
                Some("brillig"),
            );
            Ok((ssa, options))
        },
    )?;

    let result = inputs.exec()?;

    compare_results(&inputs, &result)
}

#[cfg(test)]
mod tests {
    use arbtest::arbtest;
    use std::time::Duration;

    use crate::targets::tests::{seed_from_env, should_ignore_on_ci};

    /// `cargo fuzz` takes a long time to ramp up the complexity.
    /// This test catches crash bugs much faster.
    ///
    /// Run it with for example:
    /// ```ignore
    /// NOIR_ARBTEST_SEED=0x6819c61400001000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz comptime_vs_brillig
    /// ```
    #[test]
    fn fuzz_with_arbtest() {
        if should_ignore_on_ci() {
            return;
        }
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
