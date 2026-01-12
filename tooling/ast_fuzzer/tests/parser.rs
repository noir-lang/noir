//! Test that the SSA of an arbitrary program can be printed and parsed back.
//!
//! ```shell
//! cargo test -p noir_ast_fuzzer --test parser
//! ```
use std::time::Duration;

use arbtest::arbtest;
use noir_ast_fuzzer::{Config, DisplayAstAsNoir, arb_program};
use noirc_evaluator::{
    brillig::BrilligOptions,
    ssa::{
        self,
        opt::{CONSTANT_FOLDING_MAX_ITER, INLINING_MAX_INSTRUCTIONS},
        primary_passes,
        ssa_gen::{self, Ssa},
    },
};

fn seed_from_env() -> Option<u64> {
    let Ok(seed) = std::env::var("NOIR_AST_FUZZER_SEED") else { return None };
    let seed = u64::from_str_radix(seed.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("failed to parse seed '{seed}': {e}"));
    Some(seed)
}

#[test]
fn arb_ssa_roundtrip() {
    let maybe_seed = seed_from_env();

    let mut prop = arbtest(|u| {
        let config = Config::default();
        let program = arb_program(u, config)?;

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
        let pipeline = primary_passes(&options);
        let last_pass = u.choose_index(pipeline.len())?;
        let passes = &pipeline[0..last_pass];

        // Print the AST if something goes wrong, then panic.
        let print_ast_and_panic = |msg: &str| -> ! {
            eprintln!("{}", DisplayAstAsNoir(&program));
            panic!("{msg}")
        };

        // If we have a seed to debug and we know it's going to crash, print the AST.
        if maybe_seed.is_some() {
            eprintln!("{}", DisplayAstAsNoir(&program));
        }

        // Generate the initial SSA;
        let ssa = ssa_gen::generate_ssa(program.clone()).unwrap_or_else(|e| {
            print_ast_and_panic(&format!("Failed to generate initial SSA: {e}"))
        });

        let mut ssa1 = passes.iter().fold(ssa, |ssa, pass| {
            pass.run(ssa).unwrap_or_else(|e| {
                print_ast_and_panic(&format!("Failed to run pass {}: {e}", pass.msg()))
            })
        });

        // Normalize before printing so IDs don't change.
        ssa1.normalize_ids();

        // Print to str and parse back.
        let mut ssa2 = Ssa::from_str_no_validation(&ssa1.print_without_locations().to_string())
            .unwrap_or_else(|e| {
                let msg = passes.last().map(|p| p.msg()).unwrap_or("Initial SSA");
                print_ast_and_panic(&format!(
                    "Could not parse SSA after step {last_pass} ({msg}): \n{e:?}"
                ))
            });

        ssa2.normalize_ids();

        // Not everything is populated by the parser, and unfortunately serializing to JSON doesn't work either.
        for (func_id, func1) in ssa1.functions {
            if func1.name() == "apply_dummy" {
                // The dummy function has different IDs for its parameters. But it's empty, so ignore it.
                continue;
            }
            let func2 = &ssa2.functions[&func_id];
            let values1 = func1.view().values_iter().collect::<Vec<_>>();
            let values2 = func2.view().values_iter().collect::<Vec<_>>();
            similar_asserts::assert_eq!(values1, values2);
        }

        Ok(())
    })
    .budget(Duration::from_secs(10))
    .size_min(1 << 12)
    .size_max(1 << 20);

    if let Some(seed) = maybe_seed {
        prop = prop.seed(seed);
    }

    prop.run();
}
