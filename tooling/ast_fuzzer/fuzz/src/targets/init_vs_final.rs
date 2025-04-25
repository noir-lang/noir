//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
use crate::{
    compare_results, create_ssa_or_die, create_ssa_with_passes_or_die, default_ssa_options,
};
use arbitrary::Unstructured;
use color_eyre::eyre;
use noir_ast_fuzzer::compare::ComparePasses;
use noir_ast_fuzzer::{Config, compare::CompareResult};
use noirc_evaluator::ssa::minimal_passes;

pub fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
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
