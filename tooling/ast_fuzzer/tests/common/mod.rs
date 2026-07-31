//! Shared harness for the `arbtest`-based fuzz tests in this crate.
//!
//! This mirrors the `fuzz_with_arbtest` logic the `noir_ast_fuzzer_fuzz` targets
//! use (see `tooling/ast_fuzzer/fuzz/src/targets/mod.rs`): reproduce a single
//! seed when one is given, drive the seeds from a deterministic RNG on CI so the
//! tests don't flake, and explore freely up to a time budget when running
//! locally.

use std::time::Duration;

use arbitrary::Unstructured;
use arbtest::arbtest;
use proptest::prelude::*;

/// The `Unstructured` input sizes to explore.
const MIN_SIZE: u32 = 1 << 12;
const MAX_SIZE: u32 = 1 << 20;

/// How long to run for when exploring non-deterministically (i.e. locally).
const BUDGET: Duration = Duration::from_secs(10);

/// Read a fixed seed to reproduce from the environment, if present.
pub(crate) fn seed_from_env() -> Option<u64> {
    let Ok(seed) = std::env::var("NOIR_AST_FUZZER_SEED") else { return None };
    let seed = u64::from_str_radix(seed.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("failed to parse seed '{seed}': {e}"));
    Some(seed)
}

fn bool_from_env(key: &str) -> bool {
    std::env::var(key).is_ok_and(|s| matches!(s.as_str(), "1" | "true" | "yes"))
}

/// Check if we are running on CI.
fn is_running_in_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// Check if we explicitly want non-deterministic behavior, even on CI.
fn force_non_deterministic() -> bool {
    bool_from_env("NOIR_AST_FUZZER_FORCE_NON_DETERMINISTIC")
}

/// Run an `arbtest` property, choosing the run mode the same way the
/// `noir_ast_fuzzer_fuzz` targets do:
///
/// * `NOIR_AST_FUZZER_SEED` set → reproduce that single seed.
/// * on CI (and not `NOIR_AST_FUZZER_FORCE_NON_DETERMINISTIC`) → run `cases`
///   seeds from a deterministic RNG so the test can't flake.
/// * otherwise → explore new programs until the time budget elapses.
pub(crate) fn run_fuzz(f: impl Fn(&mut Unstructured) -> arbitrary::Result<()>, cases: u32) {
    if let Some(seed) = seed_from_env() {
        arbtest(|u| f(u)).seed(seed).run();
    } else if is_running_in_ci() && !force_non_deterministic() {
        run_deterministic(f, cases);
    } else {
        arbtest(|u| f(u)).budget(BUDGET).size_min(MIN_SIZE).size_max(MAX_SIZE).run();
    }
}

/// Run a fixed number of cases with a deterministic RNG.
fn run_deterministic(f: impl Fn(&mut Unstructured) -> arbitrary::Result<()>, cases: u32) {
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
            arbtest(|u| f(u)).seed(seed).run();
            Ok(())
        })
        .unwrap();
}

/// Generate seeds for `arbtest` where the top 32 bits are random and the lower 32 bits represent the input size.
fn seed_strategy() -> BoxedStrategy<u64> {
    (MIN_SIZE..MAX_SIZE)
        .prop_flat_map(move |size| {
            any::<u64>().prop_map(move |raw| u64::from(size) | (raw << u32::BITS))
        })
        .boxed()
}
