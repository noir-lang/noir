pub mod acir_vs_brillig;
pub mod comptime_vs_brillig;
pub mod min_vs_full;
pub mod orig_vs_morph;
pub mod pass_vs_prev;

#[cfg(test)]
mod tests {

    use std::time::{Duration, Instant};

    use arbitrary::Unstructured;
    use color_eyre::eyre;
    use proptest::prelude::*;

    pub fn seed_from_env() -> Option<u64> {
        let Ok(seed) = std::env::var("NOIR_ARBTEST_SEED") else { return None };
        let seed = u64::from_str_radix(seed.trim_start_matches("0x"), 16)
            .unwrap_or_else(|e| panic!("failed to parse seed '{seed}': {e}"));
        Some(seed)
    }

    /// We can use this to disable the proptests on CI until we fix known bugs.
    ///
    /// The tests should always be enabled locally. They can be run with:
    ///
    /// ```ignore
    /// cargo test -p noir_ast_fuzzer_fuzz
    /// ```
    #[allow(unused)]
    pub fn is_running_in_ci() -> bool {
        std::env::var("CI").is_ok()
    }

    /// `cargo fuzz` takes a long time to ramp up the complexity.
    /// This test catches crash bugs much faster.
    ///
    /// Run it with for example:
    /// ```ignore
    /// NOIR_ARBTEST_SEED=0x6819c61400001000 \
    /// NOIR_AST_FUZZER_SHOW_AST=1 \
    /// cargo test -p noir_ast_fuzzer_fuzz acir_vs_brillig
    /// ```
    pub fn fuzz_with_arbtest(f: impl Fn(&mut Unstructured) -> eyre::Result<()>) {
        let _ = env_logger::try_init();

        if let Some(seed) = seed_from_env() {
            run_reproduce(f, seed);
        } else {
            run_deterministic(f);
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

    /// Run multiple tests with a deterministic RNG.
    fn run_deterministic(f: impl Fn(&mut Unstructured) -> eyre::Result<()>) {
        // Comptime tests run slower than others.
        let start = Instant::now();
        let timeout = Duration::from_secs(20);

        let config = proptest::test_runner::Config {
            cases: 1000,
            failure_persistence: None,
            ..Default::default()
        };
        let rng = proptest::test_runner::TestRng::deterministic_rng(config.rng_algorithm);
        let mut runner = proptest::test_runner::TestRunner::new_with_rng(config, rng);

        runner
            .run(&seed_strategy(), |seed| {
                if start.elapsed() < timeout {
                    run_reproduce(&f, seed);
                }
                Ok(())
            })
            .unwrap();
    }

    /// Generate seeds for `arbtest` where the top 32 bits are random and the lower 32 bits represent the input size.
    fn seed_strategy() -> proptest::strategy::BoxedStrategy<u64> {
        let min_size: u32 = 1 << 12;
        let max_size: u32 = 1 << 20;
        (min_size..max_size)
            .prop_flat_map(move |size| {
                any::<u64>().prop_map(move |raw| (size as u64) | (raw << u32::BITS))
            })
            .boxed()
    }
}
