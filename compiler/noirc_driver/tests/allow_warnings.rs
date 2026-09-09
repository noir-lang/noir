//! Integration tests for silencing backend/SSA warnings with a scoped `#[allow(...)]`.
//!
//! The `constant_return` warning is produced during ACIR generation, long after source
//! attributes are available, so honoring an `#[allow(constant_return)]` requires carrying
//! the attribute through monomorphization and SSA generation down to the ACIR entry point
//! that would emit the warning. These tests exercise that path end-to-end.

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
fn allow_constant_return_on_main_silences_a_constant_from_an_inlined_helper() {
    // The constant flows out of an inlined helper, but the return belongs to `main`,
    // so annotating `main` is what silences the warning.
    let source = r#"
    fn helper() -> Field { 1 }

    #[allow(constant_return)]
    fn main() -> pub Field { helper() }
    "#;
    let warnings = compile_warnings(source);
    assert!(warnings.is_empty(), "expected no warnings, got {warnings:?}");
}

#[test]
fn fold_function_with_constant_return_warns() {
    let source = r#"
    #[fold]
    fn folded() -> Field { 1 }

    fn main(x: Field) -> pub Field { folded() + x }
    "#;
    let warnings = compile_warnings(source);
    assert!(
        warnings.iter().any(|warning| warning.message.contains("constant")),
        "expected a constant_return warning from the fold function, got {warnings:?}"
    );
}

#[test]
fn allow_constant_return_on_fold_function_silences_its_warning() {
    // `#[fold]` functions are separate ACIR entry points and warn independently of `main`,
    // so the attribute must silence the warning when placed on the fold function itself.
    let source = r#"
    #[fold]
    #[allow(constant_return)]
    fn folded() -> Field { 1 }

    fn main(x: Field) -> pub Field { folded() + x }
    "#;
    let warnings = compile_warnings(source);
    assert!(warnings.is_empty(), "expected no warnings, got {warnings:?}");
}

#[test]
fn allow_constant_return_on_main_does_not_silence_a_fold_function() {
    // The attribute is scoped to the annotated function, so annotating `main` must not
    // leak into the fold function's own entry point.
    let source = r#"
    #[fold]
    fn folded() -> Field { 1 }

    #[allow(constant_return)]
    fn main(x: Field) -> pub Field { folded() + x }
    "#;
    let warnings = compile_warnings(source);
    assert!(
        warnings.iter().any(|warning| warning.message.contains("constant")),
        "expected the fold function's constant_return warning to survive, got {warnings:?}"
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
