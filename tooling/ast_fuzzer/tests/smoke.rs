//! Smoke test for the AST fuzzer, which generates a bunch of
//! random programs and executes them, without asserting anything
//! about the outcome. The only criteria it needs to pass is not
//! to crash the compiler, which could indicate invalid input.
//!
//! ```shell
//! cargo test -p noir_ast_fuzzer --test smoke
//! ```
use std::time::Duration;

use acir::circuit::ExpressionWidth;
use arbtest::arbtest;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::{NargoError, foreign_calls::DefaultForeignCallBuilder};
use noir_ast_fuzzer::{Config, DisplayAstAsNoir, arb_inputs, arb_program, program_abi};
use noirc_evaluator::{brillig::BrilligOptions, ssa};

#[test]
fn arb_program_can_be_executed() {
    arbtest(|u| {
        let program = arb_program(u, Config::default())?;
        let abi = program_abi(&program);

        let options = ssa::SsaEvaluatorOptions {
            ssa_logging: ssa::SsaLogging::None,
            brillig_options: BrilligOptions::default(),
            print_codegen_timings: false,
            expression_width: ExpressionWidth::default(),
            emit_ssa: None,
            skip_underconstrained_check: true,
            skip_brillig_constraints_check: true,
            enable_brillig_constraints_check_lookback: false,
            inliner_aggressiveness: 0,
            max_bytecode_increase_percent: None,
        };

        // Print the AST if something goes wrong, then panic.
        let print_ast_and_panic = |msg: &str| -> ! {
            eprintln!("{}", DisplayAstAsNoir(&program));
            panic!("{msg}")
        };

        // If we have a seed to debug and we know it's going to crash, print the AST.
        // eprintln!("{program}");

        let ssa = ssa::create_program(program.clone(), &options)
            .unwrap_or_else(|e| print_ast_and_panic(&format!("Failed to compile program: {e}")));

        let inputs = arb_inputs(u, &ssa.program, &abi)?;

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
    // .seed(0x1796975f00100000) // Uncomment and paste the seed printed by arbtest to debug a failure.
    .budget(Duration::from_secs(10))
    .size_min(1 << 12)
    .size_max(1 << 20);
}
