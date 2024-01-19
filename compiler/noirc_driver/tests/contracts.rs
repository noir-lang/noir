use std::path::Path;

use fm::FileId;
use noirc_driver::{file_manager_with_stdlib, prepare_crate, CompileOptions, ErrorsAndWarnings};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::{def_map::parse_file, Context};

#[test]
fn reject_crates_containing_multiple_contracts() -> Result<(), ErrorsAndWarnings> {
    // We use a minimal source file so that if stdlib produces warnings then we can expect these warnings to _always_
    // be emitted.
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

    let errors =
        noirc_driver::compile_contract(&mut context, root_crate_id, &CompileOptions::default())
            .unwrap_err();

    assert_eq!(
        errors,
        vec![CustomDiagnostic::from_message("Packages are limited to a single contract")
            .in_file(FileId::default())],
        "stdlib is producing warnings"
    );

    Ok(())
}
