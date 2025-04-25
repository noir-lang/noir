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

    /// `cargo fuzz` takes a long time to ramp up the complexity.
    /// This test catches crash bugs much faster.
    #[test]
    fn fuzz_with_arbtest() {
        let prop = arbtest(|u| {
            super::fuzz(u).unwrap();
            Ok(())
        })
        .budget(Duration::from_secs(10))
        .size_min(1 << 12)
        .size_max(1 << 20);

        prop.run();
    }
}
