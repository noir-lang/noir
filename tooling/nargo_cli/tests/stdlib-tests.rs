use std::{collections::BTreeMap, path::PathBuf};

use acvm::blackbox_solver::StubbedBlackBoxSolver;
use noirc_driver::{check_crate, file_manager_with_stdlib, CompileOptions};
use noirc_frontend::hir::FunctionNameMatch;

use nargo::{
    ops::{report_errors, run_test, TestStatus},
    package::{Package, PackageType},
    parse_all, prepare_package,
};

#[test]
fn run_stdlib_tests() {
    let mut file_manager = file_manager_with_stdlib(&PathBuf::from("."));
    file_manager.add_file_with_source_canonical_path(&PathBuf::from("main.nr"), "".to_owned());
    let parsed_files = parse_all(&file_manager);

    // We need a dummy package as we cannot compile the stdlib on its own.
    let dummy_package = Package {
        version: None,
        compiler_required_version: None,
        root_dir: PathBuf::from("."),
        package_type: PackageType::Binary,
        entry_path: PathBuf::from("main.nr"),
        name: "dummy".parse().unwrap(),
        dependencies: BTreeMap::new(),
    };

    let (mut context, dummy_crate_id) =
        prepare_package(&file_manager, &parsed_files, &dummy_package);

    let result = check_crate(&mut context, dummy_crate_id, true, false, false);
    report_errors(result, &context.file_manager, true, false)
        .expect("Error encountered while compiling standard library");

    // We can now search within the stdlib for any test functions to compile.

    let test_functions = context.get_all_test_functions_in_crate_matching(
        context.stdlib_crate_id(),
        FunctionNameMatch::Anything,
    );

    let test_report: Vec<(String, TestStatus)> = test_functions
        .into_iter()
        .map(|(test_name, test_function)| {
            let status = run_test(
                &StubbedBlackBoxSolver,
                &mut context,
                &test_function,
                false,
                None,
                &CompileOptions::default(),
            );

            (test_name, status)
        })
        .collect();

    assert!(!test_report.is_empty(), "Could not find any tests within the stdlib");
    assert!(test_report.iter().all(|(_, status)| !status.failed()));
}
