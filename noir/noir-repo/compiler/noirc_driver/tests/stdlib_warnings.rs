use std::path::Path;

use noirc_driver::{file_manager_with_stdlib, prepare_crate, ErrorsAndWarnings};
use noirc_frontend::hir::{def_map::parse_file, Context};

#[test]
fn stdlib_does_not_produce_constant_warnings() -> Result<(), ErrorsAndWarnings> {
    // We use a minimal source file so that if stdlib produces warnings then we can expect these warnings to _always_
    // be emitted.
    let source = "fn main() {}";

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

    let ((), warnings) = noirc_driver::check_crate(&mut context, root_crate_id, false, false)?;

    assert_eq!(warnings, Vec::new(), "stdlib is producing warnings");

    Ok(())
}
