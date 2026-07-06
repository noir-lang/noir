//! Smoke test for the AST fuzzer, which generates a bunch of
//! random programs and executes them, without asserting anything
//! about the outcome. The only criteria it needs to pass is not
//! to crash the compiler, which could indicate invalid input.
//!
//! ```shell
//! cargo test -p noir_ast_fuzzer --test smoke
//! ```
use std::time::Duration;

use arbitrary::Unstructured;
use arbtest::arbtest;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::{NargoError, foreign_calls::DefaultForeignCallBuilder};
use noir_ast_fuzzer::{Config, DisplayAstAsNoir, arb_inputs, arb_program, program_abi};
use noirc_abi::input_parser::Format;
use noirc_evaluator::ssa;
use proptest::prelude::*;

/// The `Unstructured` input sizes to explore, matching the other fuzz tests.
const MIN_SIZE: u32 = 1 << 12;
const MAX_SIZE: u32 = 1 << 20;

/// How long to run for when we explore non-deterministically (i.e. locally).
const BUDGET: Duration = Duration::from_secs(10);

/// How many programs to generate on CI, where we use a deterministic RNG.
///
/// Tune this so that CI can get through all cases in a time comparable to the
/// [`BUDGET`] used for non-deterministic local runs.
const CI_CASES: u32 = 1000;

fn seed_from_env() -> Option<u64> {
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

#[test]
fn arb_program_can_be_executed() {
    let maybe_seed = seed_from_env();

    // The property that must hold for every generated program: it has to compile
    // and execute without crashing the compiler.
    let run = |u: &mut Unstructured| -> arbitrary::Result<()> {
        let config = Config::default();
        let program = arb_program(u, config)?;
        let abi = program_abi(&program);

        let options = ssa::SsaEvaluatorOptions::default();

        // Print the AST if something goes wrong, then panic.
        let print_ast_and_panic = |msg: &str| -> ! {
            if maybe_seed.is_none() {
                eprintln!("{}", DisplayAstAsNoir(&program));
            }
            panic!("{msg}")
        };

        // If we have a seed to debug and we know it's going to crash, print the AST.
        if maybe_seed.is_some() {
            eprintln!("{}", DisplayAstAsNoir(&program));
        }

        let ssa = ssa::create_program(program.clone(), &options, None)
            .unwrap_or_else(|e| print_ast_and_panic(&format!("Failed to compile program: {e}")));

        let inputs = arb_inputs(u, &ssa.program, &abi)?;

        // It could be useful to also show the input, although in the smoke test we're mostly interested in compiler crashes,
        // not the execution. For that we have the actual fuzz targets.
        if maybe_seed.is_some() {
            eprintln!(
                "--- Inputs:\n{}",
                Format::Toml
                    .serialize(&inputs, &abi)
                    .unwrap_or_else(|e| format!("failed to serialize inputs: {e}"))
            );
        }

        let blackbox_solver = Bn254BlackBoxSolver;
        let initial_witness = abi.encode(&inputs, None).unwrap();

        let mut foreign_call_executor =
            DefaultForeignCallBuilder::default().with_mocks(false).build();

        let res = nargo::ops::execute_program(
            &ssa.program,
            initial_witness,
            &blackbox_solver,
            &mut foreign_call_executor,
        );

        match res {
            Err(NargoError::CompilationError) => {
                print_ast_and_panic("Failed to compile program into ACIR.")
            }
            Err(NargoError::ForeignCallError(e)) => {
                print_ast_and_panic(&format!("Failed to call foreign function: {e}"))
            }
            Err(NargoError::ExecutionError(_)) | Ok(_) => {
                // If some assertion failed, it's okay, we can't tell if it should or shouldn't.
                Ok(())
            }
        }
    };

    if let Some(seed) = maybe_seed {
        // Reproduce a single failing seed.
        arbtest(run).seed(seed).run();
    } else if is_running_in_ci() && !force_non_deterministic() {
        // On CI, drive the seeds from a deterministic RNG so the test explores the
        // same set of programs on every run and doesn't flake.
        run_deterministic(run, CI_CASES);
    } else {
        // Locally (or when non-determinism is explicitly requested) keep exploring
        // new programs until the time budget runs out.
        arbtest(run).budget(BUDGET).size_min(MIN_SIZE).size_max(MAX_SIZE).run();
    }
}

/// Run a fixed number of cases with a deterministic RNG, mirroring the behavior
/// of the `noir_ast_fuzzer_fuzz` targets on CI.
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
