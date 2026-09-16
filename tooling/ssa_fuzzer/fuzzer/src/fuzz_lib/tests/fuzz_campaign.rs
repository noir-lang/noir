//! A bounded run of the `acir_vs_brillig` campaign, driven the way libFuzzer drives
//! it: pick a case from the corpus, mutate it, write it back, read it back, and run the
//! program under both runtimes.
//!
//! This covers what the per-case tests cannot. `fuzz_target` panics when ACIR and
//! Brillig disagree, so every case is a comparison. And a campaign that compiles
//! nothing — because the mutator and the target no longer agree on the corpus format,
//! because mutation stopped producing programs, or because generated SSA is rejected
//! wholesale — is a campaign that reports success while comparing nothing, which the
//! floor on `programs` catches.
use crate::corpus::CorpusCodec;
use crate::fuzz_target_lib::fuzz_target;
use crate::fuzzer::FuzzerData;
use crate::mutations::mutate;
use crate::options::{FuzzerOptions, InstructionOptions};
use crate::tests::common::default_runtimes;
use rand::{RngExt, SeedableRng, rngs::StdRng};
use std::time::{Duration, Instant};

/// Cases to run when no budget is given. Each takes a few milliseconds, so this is a
/// handful of seconds of a CI partition.
const CASES: usize = 1024;

/// Seed used when `NOIR_SSA_FUZZER_SEED` is unset, so a CI run is reproducible.
const DEFAULT_SEED: u64 = 0x5ea1_5eed;

/// One case in this many must build a program. A floor rather than an exact count: the
/// rate moves with every change to the mutator, the generator and the SSA validator,
/// while what this guards against is a campaign that compares nothing at all.
const MIN_PROGRAM_RATIO: usize = 4;

/// Cases kept to mutate further. A campaign mutates entries of a corpus rather than one
/// ever-growing case, and a single chain degrades: it drifts into states that describe
/// no program and every later mutation starts from there.
const MAX_CORPUS_SIZE: usize = 32;

/// Seed for the mutator, overridable to replay a failure a nightly run reports.
fn seed() -> u64 {
    let Ok(seed) = std::env::var("NOIR_SSA_FUZZER_SEED") else { return DEFAULT_SEED };
    let trimmed = seed.trim_start_matches("0x");
    u64::from_str_radix(trimmed, 16)
        .unwrap_or_else(|error| panic!("failed to parse seed '{seed}': {error}"))
}

/// How long to keep mutating. Unset means a fixed number of cases instead, which keeps
/// the test deterministic for CI; a nightly run sets it to explore further.
fn budget() -> Option<Duration> {
    let budget = std::env::var("NOIR_SSA_FUZZER_BUDGET_SECS").ok()?;
    let secs =
        budget.parse().unwrap_or_else(|error| panic!("failed to parse budget '{budget}': {error}"));
    Some(Duration::from_secs(secs))
}

/// The options of the `acir_vs_brillig` target, so this runs the configuration the
/// campaign runs. Unsafe get/set stays disabled because ACIR and Brillig disagree on
/// it: https://github.com/noir-lang/noir/issues/9159
fn campaign_options() -> FuzzerOptions {
    FuzzerOptions {
        instruction_options: InstructionOptions {
            unsafe_get_set_enabled: false,
            ..InstructionOptions::default()
        },
        ..FuzzerOptions::default()
    }
}

#[test]
fn mutated_cases_reach_both_runtimes() {
    let _ = env_logger::try_init();

    let seed = seed();
    let budget = budget();
    let codec = CorpusCodec::Json;
    let options = campaign_options();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut corpus = vec![FuzzerData::default()];
    let mut cases = 0usize;
    let mut programs = 0usize;
    let start = Instant::now();

    while budget.map_or(cases < CASES, |budget| start.elapsed() < budget) {
        let mut case = corpus[rng.random_range(0..corpus.len())].clone();
        mutate(&mut case, &mut rng);

        // The mutator writes the case to the corpus and the target reads it back, so the
        // program that runs is the one the bytes describe, not whatever a failed decode
        // left behind.
        let entry = codec.encode(&case);
        let case = codec.decode(&entry).unwrap_or_else(|error| {
            panic!("case {cases} (seed {seed:#x}) did not survive the corpus: {error}")
        });

        // Panics if the runtimes disagree on the return values or on which of them failed.
        let output = fuzz_target(case.clone(), default_runtimes(), options.clone());
        if output.program.is_some() {
            programs += 1;
            if corpus.len() < MAX_CORPUS_SIZE {
                corpus.push(case);
            } else {
                corpus[rng.random_range(0..MAX_CORPUS_SIZE)] = case;
            }
        }
        cases += 1;
    }

    println!("{programs}/{cases} mutated cases built a program (seed {seed:#x})");
    assert!(
        programs * MIN_PROGRAM_RATIO >= cases,
        "only {programs} of {cases} mutated cases built a program (seed {seed:#x}); \
         the campaign is not exercising the ACIR/Brillig comparison"
    );
}
