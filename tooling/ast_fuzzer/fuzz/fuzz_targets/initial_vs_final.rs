//! Compare the execution of random ASTs between the initial SSA
//! (or as close as we can stay to the initial state)
//! and the fully optimized version.
#![no_main]

use acir::circuit::ExpressionWidth;
use color_eyre::eyre::{self, Context, bail};
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use noir_ast_fuzzer::Config;
use noir_ast_fuzzer::compare::{ComparePasses, CompareResult};
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

    // TODO: What we really want is to do the minimum number of passes on the SSA to leave it as close to the initial SSA as possible.
    // For now just test with min/max inliner aggressiveness.
    let inputs = ComparePasses::arb(
        u,
        Config::default(),
        |p| {
            ssa::create_program(
                p.clone(),
                &ssa::SsaEvaluatorOptions { inliner_aggressiveness: i64::MIN, ..options.clone() },
            )
            .expect("create_program 1")
        },
        |p| {
            ssa::create_program(
                p.clone(),
                &ssa::SsaEvaluatorOptions { inliner_aggressiveness: i64::MAX, ..options.clone() },
            )
            .expect("create_program 2")
        },
    )?;

    let result = inputs.exec().wrap_err("exec")?;

    match result {
        CompareResult::BothFailed(_, _) => Ok(()),
        CompareResult::LeftFailed(e, _) => {
            bail!("first program failed: {e}")
        }
        CompareResult::RightFailed(_, e) => {
            bail!("second program failed: {e}")
        }
        CompareResult::Disagree(r1, r2) => {
            bail!("programs disagree: {r1:?} != {r2:?}")
        }
        CompareResult::Agree(_) => Ok(()),
    }
}
