use std::path::Path;

use fm::FileId;
use noirc_driver::{CompileOptions, ErrorsAndWarnings, file_manager_with_stdlib, prepare_crate};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::{Context, def_map::parse_file};

#[test]
fn reject_crates_without_enable_contracts_flag() -> Result<(), ErrorsAndWarnings> {
    let source = "contract Foo {}";

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

    let errors =
        noirc_driver::compile_contract(&mut context, root_crate_id, &CompileOptions::default())
            .unwrap_err();

    assert_eq!(
        errors,
        vec![CustomDiagnostic::from_message(
            "compiling contract crates is disabled by default. To enable, pass the `--enable-contracts` flag to Nargo.",
            FileId::default()
        )],
        "compiler should reject contract crates when the enable_contracts flag is not set"
    );

    Ok(())
}

#[test]
fn reject_crates_containing_multiple_contracts() -> Result<(), ErrorsAndWarnings> {
    let source = "
contract Foo {}

contract Bar {}";

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

    let options = CompileOptions { enable_contracts: true, ..CompileOptions::default() };
    let errors = noirc_driver::compile_contract(&mut context, root_crate_id, &options).unwrap_err();

    assert_eq!(
        errors,
        vec![CustomDiagnostic::from_message(
            "Packages are limited to a single contract",
            FileId::default()
        )],
        "compiler should reject crates containing multiple contracts"
    );

    Ok(())
}
