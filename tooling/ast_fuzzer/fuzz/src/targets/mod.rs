use arbitrary::Unstructured;
use noir_ast_fuzzer::Config;

pub mod acir_vs_brillig;
pub mod comptime_vs_brillig_direct;
pub mod comptime_vs_brillig_nargo;
pub mod min_vs_full;
pub mod orig_vs_morph;
pub mod pass_vs_prev;

/// Create a default configuration instance, with some common flags randomized.
fn default_config(u: &mut Unstructured) -> arbitrary::Result<Config> {
    // Some errors such as overflows and OOB are easy to trigger, so in half
    // the cases we avoid all of them, to make sure they don't mask other errors.
    let avoid_frequent_errors = u.arbitrary()?;
    let config = Config {
        avoid_overflow: avoid_frequent_errors,
        avoid_index_out_of_bounds: avoid_frequent_errors,
        ..Default::default()
    };
    Ok(config)
}

/// Common functions used in the test modules of targets.
#[cfg(test)]
mod tests {

    const BUDGET: Duration = Duration::from_secs(20);
    const MIN_SIZE: u32 = 1 << 12;
    const MAX_SIZE: u32 = 1 << 20;

    use std::time::Duration;

    use arbitrary::Unstructured;
    use color_eyre::eyre;
    use proptest::prelude::*;

    use crate::bool_from_env;

    fn seed_from_env() -> Option<u64> {
        let Ok(seed) = std::env::var("NOIR_AST_FUZZER_SEED") else { return None };
        let seed = u64::from_str_radix(seed.trim_start_matches("0x"), 16)
            .unwrap_or_else(|e| panic!("failed to parse seed '{seed}': {e}"));
        Some(seed)
    }

    /// How long to let non-deterministic tests run for.
    fn budget() -> Duration {
        std::env::var("NOIR_AST_FUZZER_BUDGET_SECS").ok().map_or(BUDGET, |b| {
            Duration::from_secs(
                b.parse().unwrap_or_else(|e| panic!("failed to parse budget; got {b}: {e}")),
            )
        })
    }

    /// Check if we are running on CI.
    fn is_running_in_ci() -> bool {
        std::env::var("CI").is_ok()
    }

    /// Check if we explicitly want non-deterministic behavior, even on CI.
    fn force_non_deterministic() -> bool {
        bool_from_env("NOIR_AST_FUZZER_FORCE_NON_DETERMINISTIC")
    }

    /// `cargo fuzz` takes a long time to ramp up the complexity.
    /// This test catches crash bugs much faster.
    ///
    /// Run it with for example:
    /// ```ignore
    /// NOIR_AST_FUZZER_SEED=0x6819c61400001000 \
    /// cargo test -p noir_ast_fuzzer_fuzz acir_vs_brillig
    /// ```
    ///
    /// The `cases` determine how many tests to run on CI.
    /// Tune this so that we can expect CI to be able to get through all cases in reasonable time.
    pub fn fuzz_with_arbtest(f: impl Fn(&mut Unstructured) -> eyre::Result<()>, cases: u32) {
        let _ = env_logger::try_init();

        if let Some(seed) = seed_from_env() {
            run_reproduce(f, seed);
        } else if is_running_in_ci() && !force_non_deterministic() {
            run_deterministic(f, cases);
        } else {
            run_nondeterministic(f);
        }
    }

    /// Reproduce the result of a single seed.
    fn run_reproduce(f: impl Fn(&mut Unstructured) -> eyre::Result<()>, seed: u64) {
        arbtest::arbtest(|u| {
            f(u).unwrap();
            Ok(())
        })
        .seed(seed)
        .run();
    }

    /// Run the tests non-deterministically until the timeout.
    ///
    /// This is the local behavior.
    fn run_nondeterministic(f: impl Fn(&mut Unstructured) -> eyre::Result<()>) {
        arbtest::arbtest(|u| {
            f(u).unwrap();
            Ok(())
        })
        .size_min(MIN_SIZE)
        .size_max(MAX_SIZE)
        .budget(budget())
        .run();
    }

    /// Run multiple tests with a deterministic RNG.
    ///
    /// This is the behavior on CI.
    fn run_deterministic(f: impl Fn(&mut Unstructured) -> eyre::Result<()>, cases: u32) {
        let config = proptest::test_runner::Config {
            cases,
            failure_persistence: None,
            max_shrink_iters: 0,
            ..Default::default()
        };
        let rng = proptest::test_runner::TestRng::deterministic_rng(config.rng_algorithm);
        let mut runner = proptest::test_runner::TestRunner::new_with_rng(config, rng);

        runner
            .run(&seed_strategy(), |seed| {
                run_reproduce(&f, seed);
                Ok(())
            })
            .unwrap();
    }

    /// Generate seeds for `arbtest` where the top 32 bits are random and the lower 32 bits represent the input size.
    fn seed_strategy() -> proptest::strategy::BoxedStrategy<u64> {
        (MIN_SIZE..MAX_SIZE)
            .prop_flat_map(move |size| {
                any::<u64>().prop_map(move |raw| (size as u64) | (raw << u32::BITS))
            })
            .boxed()
    }
}
