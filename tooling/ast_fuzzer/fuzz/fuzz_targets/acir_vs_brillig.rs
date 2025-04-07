//! Compare the execution of random ASTs between the normal execution
//! vs when everything is forced to be Brillig.

#![no_main]

use acir::circuit::ExpressionWidth;
use color_eyre::eyre::{self, Context};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::CompareMutants;
use noir_ast_fuzzer_fuzz::create_ssa_or_die;
use noirc_evaluator::brillig::BrilligOptions;
use noirc_evaluator::ssa;

fuzz_target!(|data: &[u8]| {
    fuzz(&mut Unstructured::new(data)).unwrap();
});

fn fuzz(u: &mut Unstructured) -> eyre::Result<()> {
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

    let inputs = CompareMutants::arb(
        u,
        Config::default(),
        |_u, program| {
            // Change every function to be unconstrained.
            let mut program = program.clone();
            for f in program.functions.iter_mut() {
                f.unconstrained = true;
            }
            Ok(program)
        },
        |program| create_ssa_or_die(program, &options, None),
    )?;

    let result = inputs.exec().wrap_err("exec")?;

    let _ = result.return_value_or_err()?;
    Ok(())
}
