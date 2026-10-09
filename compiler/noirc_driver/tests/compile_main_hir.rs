//! Integration tests for `compile_main` releasing the HIR before code generation.
//!
//! `compile_main` frees the context's `NodeInterner` once the program has been monomorphized.
//! Everything code generation needs from the HIR, including the ABI's error types, must therefore
//! be derivable without the interner.

use std::path::Path;

use noirc_abi::{AbiErrorType, AbiType};
use noirc_artifacts::program::CompiledProgram;
use noirc_driver::{CompileOptions, file_manager_with_stdlib, prepare_crate};
use noirc_frontend::hir::{Context, def_map::parse_file};

fn compile(source: &str) -> CompiledProgram {
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
    let (program, _warnings) = noirc_driver::compile_main(context, root_crate_id, &options, None)
        .expect("program should compile successfully");
    program
}

#[test]
fn abi_error_types_resolve_after_the_hir_is_released() {
    let source = r#"
        struct MyError { code: Field }

        fn main(x: Field, y: u32) {
            assert(x != 0, MyError { code: x });
            assert(y != 1, f"bad value {y}");
        }
    "#;
    let program = compile(source);

    let error_types: Vec<&AbiErrorType> = program.abi.error_types.values().collect();
    assert!(
        error_types.iter().any(|error_type| matches!(
            error_type,
            AbiErrorType::Custom(AbiType::Struct { path, fields })
                if path == "MyError" && fields.len() == 1
        )),
        "expected a custom `MyError` error type in the ABI, got {error_types:?}"
    );
    assert!(
        error_types.iter().any(|error_type| matches!(
            error_type,
            AbiErrorType::FmtString { item_types, .. } if item_types.len() == 1
        )),
        "expected a format string error type in the ABI, got {error_types:?}"
    );
}
