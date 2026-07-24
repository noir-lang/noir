//! Integration tests for silencing backend/SSA warnings with a scoped `#[allow(...)]`.
//!
//! The `constant_return` warning is produced during ACIR generation, long after source
//! attributes are available, so honoring an `#[allow(constant_return)]` requires matching
//! the warning's call stack back to the annotated function. These tests exercise that path
//! end-to-end.

use std::path::Path;

use noirc_driver::{CompileOptions, file_manager_with_stdlib, prepare_crate};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::{Context, def_map::parse_file};

fn compile_warnings(source: &str) -> Vec<CustomDiagnostic> {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source.to_owned()).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );
    let parsed_files = file_manager
        .as_file_map()
        .all_file_ids()
        .map(|&file_id| (file_id, parse_file(&file_manager, file_id)))
        .collect();

    let mut context = Context::new(file_manager, parsed_files);
    let root_crate_id = prepare_crate(&mut context, file_name);

    let options = CompileOptions::default();
    let (_program, warnings) =
        noirc_driver::compile_main(&mut context, root_crate_id, &options, None)
            .expect("program should compile successfully");
    warnings
}

#[test]
fn constant_return_warns_without_allow() {
    let source = "fn main() -> pub Field { 1 }";
    let warnings = compile_warnings(source);
    assert!(
        warnings.iter().any(|warning| warning.message.contains("constant")),
        "expected a constant_return warning, got {warnings:?}"
    );
}

#[test]
fn allow_constant_return_silences_the_warning() {
    let source = "#[allow(constant_return)]\nfn main() -> pub Field { 1 }";
    let warnings = compile_warnings(source);
    assert!(warnings.is_empty(), "expected no warnings, got {warnings:?}");
}

#[test]
fn allow_of_a_different_lint_does_not_silence_constant_return() {
    // Silencing is keyed on the specific lint name, so an unrelated `#[allow]` must not
    // suppress the `constant_return` warning.
    let source = "#[allow(dead_code)]\nfn main() -> pub Field { 1 }";
    let warnings = compile_warnings(source);
    assert!(
        warnings.iter().any(|warning| warning.message.contains("constant")),
        "expected the constant_return warning to survive an unrelated allow, got {warnings:?}"
    );
}

#[test]
fn allow_constant_return_on_another_function_does_not_silence_main() {
    // The attribute is scoped to the annotated function's body, so annotating `helper`
    // must not leak into `main`'s constant return.
    let source = r#"
    #[allow(constant_return)]
    fn helper(x: Field) -> Field { x + 1 }

    fn main(x: Field) -> pub Field {
        assert(helper(x) == x + 1);
        1
    }
    "#;
    let warnings = compile_warnings(source);
    assert!(
        warnings.iter().any(|warning| warning.message.contains("constant")),
        "expected main's constant_return warning to survive, got {warnings:?}"
    );
}
