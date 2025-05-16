//! This crate represents utility methods which can be useful for testing in other crates
//! which also desire to compile the frontend.
//!
//! This module is split out from the `tests` module and has an additional `test_utils` feature
//! as a module configured only for tests will not be accessible in other crates.
//! A crate that needs to use the methods in this module should add the `noirc_frontend`
//! crate as a dev dependency with the `test_utils` feature activated.
#![cfg(any(test, feature = "test_utils"))]

use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;

use crate::elaborator::FrontendOptions;

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::hir::Context;
use crate::hir::def_collector::dc_crate::CompilationError;
use crate::hir::def_collector::dc_crate::DefCollector;
use crate::hir::def_map::CrateDefMap;
use crate::hir::def_map::ModuleData;
use crate::parser::{ItemKind, ParserErrorReason};
use crate::token::SecondaryAttribute;
use crate::{ParsedModule, parse_program};
use fm::FileManager;

use crate::monomorphization::{ast::Program, errors::MonomorphizationError, monomorphize};

pub fn get_monomorphized_no_emit_test(src: &str) -> Result<Program, MonomorphizationError> {
    get_monomorphized(src, None, Expect::Success)
}

pub fn get_monomorphized(
    src: &str,
    test_path: Option<&str>,
    expect: Expect,
) -> Result<Program, MonomorphizationError> {
    let (_parsed_module, mut context, errors) = get_program(src, test_path, expect);
    assert!(
        errors.iter().all(|err| !err.is_error()),
        "Expected monomorphized program to have no errors before monomorphization, but found: {errors:?}"
    );

    let main = context
        .get_main_function(context.root_crate_id())
        .unwrap_or_else(|| panic!("get_monomorphized: test program contains no 'main' function"));

    monomorphize(main, &mut context.def_interner, false)
}

pub(crate) fn has_parser_error(errors: &[CompilationError]) -> bool {
    errors.iter().any(|e| matches!(e, CompilationError::ParseError(_)))
}

pub(crate) fn remove_experimental_warnings(errors: &mut Vec<CompilationError>) {
    errors.retain(|error| match error {
        CompilationError::ParseError(error) => {
            !matches!(error.reason(), Some(ParserErrorReason::ExperimentalFeature(..)))
        }
        _ => true,
    });
}

pub(crate) fn get_program<'a, 'b>(
    src: &'a str,
    test_path: Option<&'b str>,
    expect: Expect,
) -> (ParsedModule, Context<'a, 'b>, Vec<CompilationError>) {
    let allow_parser_errors = false;
    get_program_with_options(
        src,
        test_path,
        expect,
        allow_parser_errors,
        FrontendOptions::test_default(),
    )
}

pub enum Expect {
    Bug,
    Success,
    Error,
}

/// Compile a program.
///
/// The stdlib is not available for these snippets.
///
/// An optional test path is supplied as an argument.
/// The existence of a test path indicates that we want to emit integration tests
/// for the supplied program as well.
pub(crate) fn get_program_with_options(
    src: &str,
    test_path: Option<&str>,
    expect: Expect,
    allow_parser_errors: bool,
    options: FrontendOptions,
) -> (ParsedModule, Context<'static, 'static>, Vec<CompilationError>) {
    let root = std::path::Path::new("/");
    let mut fm = FileManager::new(root);
    let root_file_id = fm.add_file_with_source(Path::new("test_file"), src.to_string()).unwrap();
    let mut context = Context::new(fm, Default::default());

    context.def_interner.populate_dummy_operator_traits();
    let root_crate_id = context.crate_graph.add_crate_root(root_file_id);

    let (program, parser_errors) = parse_program(src, root_file_id);
    let mut errors = vecmap(parser_errors, |e| e.into());
    remove_experimental_warnings(&mut errors);

    if allow_parser_errors || !has_parser_error(&errors) {
        let inner_attributes: Vec<SecondaryAttribute> = program
            .items
            .iter()
            .filter_map(|item| {
                if let ItemKind::InnerAttribute(attribute) = &item.kind {
                    Some(attribute.clone())
                } else {
                    None
                }
            })
            .collect();

        let location = Location::new(Default::default(), root_file_id);
        let root_module = ModuleData::new(
            None,
            location,
            Vec::new(),
            inner_attributes.clone(),
            false, // is contract
            false, // is struct
        );

        let def_map = CrateDefMap::new(root_crate_id, root_module);

        // Now we want to populate the CrateDefMap using the DefCollector
        errors.extend(DefCollector::collect_crate_and_dependencies(
            def_map,
            &mut context,
            program.clone().into_sorted(),
            root_file_id,
            options,
        ));
    }

    if let Some(test_path) = test_path {
        emit_compile_test(test_path, src, expect);
    }

    (program, context, errors)
}

// if the "nextest" feature is enabled, this will panic instead of emitting a test crate
fn emit_compile_test(test_path: &str, src: &str, mut expect: Expect) {
    let package_name = test_path.replace("::", "_");
    let skipped_tests = [
        // skip ~2.4k name_shadowing tests
        "name_shadowing_",
        // TODO(https://github.com/noir-lang/noir/issues/7763)
        "unconditional_recursion_fail_",
        "unconditional_recursion_pass_",
        // TODO(https://github.com/noir-lang/noir/issues/7783): array type fails to resolve when
        // compiled
        "traits_calls_trait_method_using_struct_name_when_multiple_impls_exist",
        // TODO(https://github.com/noir-lang/noir/issues/7766): trait generic that passes
        // frontend test fails to resolve with nargo
        "turbofish_numeric_generic_nested_",
    ];
    if skipped_tests.iter().any(|skipped_test_name| package_name.contains(skipped_test_name)) {
        return;
    }

    // in these cases, we expect a warning when 'check_errors' or similar is used
    let error_to_warn_cases = [
        "cast_256_to_u8_size_checks",
        "enums_errors_on_unspecified_unstable_enum",
        "imports_warns_on_use_of_private_exported_item",
        "metaprogramming_does_not_fail_to_parse_macro_on_parser_warning",
        "resolve_unused_var",
        "struct_array_len",
        "unused_items_errors_on_unused_private_import",
        "unused_items_errors_on_unused_pub_crate_import",
        "unused_items_errors_on_unused_struct",
        "unused_items_errors_on_unused_trait",
        "unused_items_errors_on_unused_type_alias",
        "unused_items_warns_on_unused_global",
        "visibility_warns_if_calling_private_struct_method",
        "warns_on_nested_unsafe",
        "warns_on_unneeded_unsafe",
        // TODO(https://github.com/noir-lang/noir/issues/6932): these will be hard errors
        "visibility_error_when_accessing_private_struct_field",
        "visibility_error_when_using_private_struct_field_in_constructor",
        "visibility_error_when_using_private_struct_field_in_struct_pattern",
        "visibility_errors_if_accessing_private_struct_member_inside_comptime_context",
        "visibility_errors_if_accessing_private_struct_member_inside_function_generated_at_comptime",
        "visibility_errors_if_trying_to_access_public_function_inside_private_module",
        "visibility_errors_once_on_unused_import_that_is_not_accessible",
        // TODO(https://github.com/noir-lang/noir/issues/7795): these will be hard errors
        "indexing_array_with_non_u32_on_lvalue_produces_a_warning",
        "indexing_array_with_non_u32_produces_a_warning",
    ];
    if let Expect::Error = expect {
        if error_to_warn_cases
            .iter()
            .any(|error_to_warn_case| package_name.contains(error_to_warn_case))
        {
            expect = Expect::Success;
        }
    }

    let error_to_bug_cases = ["cast_negative_one_to_u8_size_checks"];
    if let Expect::Success = expect {
        if error_to_bug_cases
            .iter()
            .any(|error_to_bug_case| package_name.contains(error_to_bug_case))
        {
            expect = Expect::Bug;
        }
    }

    // "compiler/noirc_frontend"
    let noirc_frontend_path = Path::new(std::env!("CARGO_MANIFEST_DIR"));
    let noir_root_path = noirc_frontend_path
        .parent()
        .expect("expected 'noirc_frontend' to be in 'compiler'")
        .parent()
        .expect("expected 'compiler' to be in the noir root");
    let test_programs_path = noir_root_path.join("test_programs");

    let tests_dir_name = match expect {
        Expect::Bug => "compile_success_with_bug",
        Expect::Success => "compile_success_no_bug",
        Expect::Error => "compile_failure",
    };
    let tests_dir = test_programs_path.join(tests_dir_name);
    let crate_path = tests_dir.join(&package_name);
    let nargo_toml_path = crate_path.join("Nargo.toml");
    let src_hash_path = crate_path.join("src_hash.txt");
    let src_path = crate_path.join("src");
    let main_nr_path = src_path.join("main.nr");

    // hash `src`
    let mut hasher = DefaultHasher::new();
    src.hash(&mut hasher);
    let new_hash = hasher.finish().to_string();

    if crate_path.is_dir() && src_hash_path.is_file() {
        let current_hash =
            std::fs::read_to_string(&src_hash_path).expect("Unable to read src_hash.txt");
        // if out of date, update main.nr and hash file
        if current_hash != new_hash {
            if cfg!(feature = "nextest") {
                panic!(
                    "test generated from frontend unit test {test_path} is out of date: run `cargo test` to update"
                );
            }
            std::fs::write(main_nr_path, src).expect("Unable to write test file");
            std::fs::write(src_hash_path, new_hash).expect("Unable to write src_hash.txt file");
        }
    } else {
        if cfg!(feature = "nextest") {
            panic!(
                "new test generated from frontend unit test {test_path}: run `cargo test` to generate"
            );
        }

        // create missing dir's
        std::fs::create_dir_all(&crate_path).unwrap_or_else(|_| {
            panic!("expected to be able to create the directory {}", crate_path.display())
        });
        std::fs::create_dir_all(&src_path).unwrap_or_else(|_| {
            panic!("expected to be able to create the directory {}", src_path.display())
        });

        let package_type = "bin"; // nargo::package::PackageType::Binary;
        let toml_contents = format!(
            r#"
            [package]
            name = "{package_name}"
            type = "{package_type}"
            authors = [""]
            
            [dependencies]"#
        );

        std::fs::write(&nargo_toml_path, toml_contents).unwrap_or_else(|_| {
            panic!("Unable to write Nargo.toml to {}", nargo_toml_path.display())
        });
        std::fs::write(&main_nr_path, src)
            .unwrap_or_else(|_| panic!("Unable to write test file to {}", main_nr_path.display()));
        std::fs::write(&src_hash_path, new_hash).unwrap_or_else(|_| {
            panic!("Unable to write src_hash.txt file to {}", src_hash_path.display())
        });
    }
}

// NOTE: this will fail in CI when called twice within one test: test names must be unique
#[macro_export]
macro_rules! function_path {
    () => {
        std::concat!(std::module_path!(), "::", function_name!(),)
    };
}
