//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
#![no_main]

use acir::circuit::ExpressionWidth;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use color_eyre::eyre::{self, Context};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use nargo::PrintOutput;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use noir_ast_fuzzer::{Config, arb_inputs, arb_program};
use noirc_evaluator::brillig::BrilligOptions;
use noirc_evaluator::ssa;

fuzz_target!(|data: &[u8]| {
    fuzz(&mut Unstructured::new(data)).unwrap();
});

fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
    let (program, abi) = arb_program(u, Config::default()).wrap_err("arb_program")?;

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

    let blackbox_solver = Bn254BlackBoxSolver(false);
    let mut foreign_call_executor = DefaultForeignCallBuilder::default()
        .with_mocks(false)
        .with_output(PrintOutput::None)
        .build();

    // TODO: What we really want is to control which SSA passes get executed, but for now just get it working.
    let ssa_program1 = ssa::create_program(
        program.clone(),
        &ssa::SsaEvaluatorOptions { inliner_aggressiveness: i64::MIN, ..options.clone() },
    )
    .wrap_err("create_program")?;

    let ssa_program2 = ssa::create_program(
        program,
        &ssa::SsaEvaluatorOptions { inliner_aggressiveness: i64::MAX, ..options },
    )
    .wrap_err("create_program")?;

    let input_map = arb_inputs(u, &ssa_program1.program, &abi).wrap_err("arb_inputs")?;

    let initial_witness = abi.encode(&input_map, None).wrap_err("abi.encode")?;

    let result1 = nargo::ops::execute_program(
        &ssa_program1.program,
        initial_witness.clone(),
        &blackbox_solver,
        &mut foreign_call_executor,
    )
    .wrap_err("execute_program")?;

    let result2 = nargo::ops::execute_program(
        &ssa_program2.program,
        initial_witness,
        &blackbox_solver,
        &mut foreign_call_executor,
    )
    .wrap_err("execute_program")?;

    assert_eq!(result1, result2, "the two versions disagree");

    Ok(())
}
