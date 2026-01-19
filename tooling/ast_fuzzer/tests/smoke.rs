//! Smoke test for the AST fuzzer, which generates a bunch of
//! random programs and executes them, without asserting anything
//! about the outcome. The only criteria it needs to pass is not
//! to crash the compiler, which could indicate invalid input.
//!
//! ```shell
//! cargo test -p noir_ast_fuzzer --test smoke
//! ```
use std::time::Duration;

use arbtest::arbtest;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::{NargoError, foreign_calls::DefaultForeignCallBuilder};
use noir_ast_fuzzer::{Config, DisplayAstAsNoir, arb_inputs, arb_program, program_abi};
use noirc_abi::input_parser::Format;
use noirc_evaluator::{
    brillig::BrilligOptions,
    ssa::{
        self,
        opt::{CONSTANT_FOLDING_MAX_ITER, INLINING_MAX_INSTRUCTIONS},
    },
};

fn seed_from_env() -> Option<u64> {
    let Ok(seed) = std::env::var("NOIR_AST_FUZZER_SEED") else { return None };
    let seed = u64::from_str_radix(seed.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("failed to parse seed '{seed}': {e}"));
    Some(seed)
}

#[test]
fn arb_program_can_be_executed() {
    let maybe_seed = seed_from_env();

    let mut prop = arbtest(|u| {
        let config = Config::default();
        let program = arb_program(u, config)?;
        let abi = program_abi(&program);

        let options = ssa::SsaEvaluatorOptions {
            ssa_logging: ssa::SsaLogging::None,
            brillig_options: BrilligOptions::default(),
            print_codegen_timings: false,
            emit_ssa: None,
            skip_underconstrained_check: true,
            skip_brillig_constraints_check: true,
            enable_brillig_constraints_check_lookback: false,
            inliner_aggressiveness: 0,
            constant_folding_max_iter: CONSTANT_FOLDING_MAX_ITER,
            small_function_max_instruction: INLINING_MAX_INSTRUCTIONS,
            max_bytecode_increase_percent: None,
            skip_passes: Default::default(),
        };

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

        let blackbox_solver = Bn254BlackBoxSolver(false);
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
    })
    .budget(Duration::from_secs(10))
    .size_min(1 << 12)
    .size_max(1 << 20);

    if let Some(seed) = maybe_seed {
        prop = prop.seed(seed);
    }

    prop.run();
}
