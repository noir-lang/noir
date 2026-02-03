//! Test that the AST of an arbitrary program can be printed, parsed, compiled, printed again,
//! and it results in the same AST. This should bring to light any assumptions the monomorphizer
//! puts in place during compilation that the direct AST generator does not respect.
//!
//! ```shell
//! cargo test -p noir_ast_fuzzer --test mono
//! ```

use std::{path::Path, time::Duration};

use arbtest::arbtest;
use nargo::parse_all;
use noir_ast_fuzzer::{Config, DisplayAstAsNoir, arb_program};
use noirc_driver::{CompileOptions, file_manager_with_stdlib, prepare_crate};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::{
    elaborator::UnstableFeature,
    hir::Context,
    monomorphization::{ast::Program, monomorphize},
};

fn seed_from_env() -> Option<u64> {
    let Ok(seed) = std::env::var("NOIR_AST_FUZZER_SEED") else { return None };
    let seed = u64::from_str_radix(seed.trim_start_matches("0x"), 16)
        .unwrap_or_else(|e| panic!("failed to parse seed '{seed}': {e}"));
    Some(seed)
}

#[test]
fn arb_ast_roundtrip() {
    let maybe_seed = seed_from_env();

    let mut prop = arbtest(|u| {
        let config = Config {
            // The monomorphizer creates proxy functions, which the AST generator skips.
            // Rather than try to match it, let's ignore prints in this test.
            avoid_print: true,
            // Negative literals can cause problems: --128_i8 is a compile error; --100_i32 is printed back as 100_i32.
            avoid_negative_int_literals: true,
            // Large ints are rejected in for loops, unless we use suffixes.
            avoid_large_int_literals: true,
            // The compiler introduces "internal variable" even if it's not needed,
            // and also rationalizes removes branches that can never be matched,
            // (like repeated patterns, superfluous defaults). For now ignore these.
            avoid_match: true,
            // Since #9484 the monomorphizer represents function values as a pair of
            // `(constrained, unconstrained)` where each element is a different runtime of the same
            // function. The fuzzer has not yet been updated to mimic this so first-class functions
            // are avoided for now.
            avoid_lambdas: true,
            // The formatting of `unsafe { ` becomes `{ unsafe {` with extra line breaks.
            // Let's stick to just Brillig so there is no need for `unsafe` at all.
            force_brillig: true,
            ..Default::default()
        };
        let program1 = arb_program(u, config)?;
        let src1 = format!("{}", DisplayAstAsNoir(&program1));
        let program2 = monomorphize_snippet(src1.clone()).unwrap_or_else(|errors| {
            panic!("the program did not compile:\n{src1}\n\n{errors:?}");
        });
        let src2 = format!("{}", DisplayAstAsNoir(&program2));
        compare_sources(&src1, &src2);
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

fn monomorphize_snippet(source: String) -> Result<Program, Vec<CustomDiagnostic>> {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );
    let parsed_files = parse_all(&file_manager);

    let mut context = Context::new(file_manager, parsed_files);
    let crate_id = prepare_crate(&mut context, file_name);

    let options = CompileOptions {
        unstable_features: vec![UnstableFeature::Enums],
        disable_comptime_printing: true,
        ..Default::default()
    };

    let _ = noirc_driver::check_crate(&mut context, crate_id, &options)?;

    let main_id = context.get_main_function(&crate_id).expect("get_main_function");

    let program = monomorphize(main_id, &mut context.def_interner, false).expect("monomorphize");

    Ok(program)
}

/// Get rid of superficial differences.
fn sanitize(src: &str) -> String {
    src
        // Sometimes `;` is removed, or duplicated.
        .replace(";", "")
        // Sometimes a unit value is printed as () other times as {}
        .replace("{}", "()")
        // Double negation is removed during parsing
        .replace("--", "")
        // Negative zero is parsed as zero
        // (NB we don't want to avoid generating -0, because there were bugs related to it).
        .replace("-0", "0")
}

fn split_functions(src: &str) -> Vec<String> {
    // Split along the closing brace of the functions.
    let sep = "\n}";
    src.split(sep).map(|f| format!("{f}{sep}")).collect()
}

fn compare_sources(src1: &str, src2: &str) {
    let prepare = |src| {
        let mut v = split_functions(src);
        let main = v.remove(0);
        // Unused globals are not rendered. Ignore all globals.
        let (_globals, main) = main.split_once("unconstrained fn main").unwrap();
        let main = format!("unconstrained fn main {main}");
        // Sort the other functions alphabetically.
        v.sort();
        sanitize(&format!("{main}\n{}", v.join("\n")))
    };
    let src1 = prepare(src1);
    let src2 = prepare(src2);
    similar_asserts::assert_eq!(src1, src2);
}
