use std::path::Path;

use fm::FileId;
use noirc_abi::{AbiType, AbiValue};
use noirc_driver::{CompileOptions, ErrorsAndWarnings, file_manager_with_stdlib, prepare_crate};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::{Context, def_map::parse_file};

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

    let errors =
        noirc_driver::compile_contract(&mut context, root_crate_id, &CompileOptions::default())
            .unwrap_err();

    assert_eq!(
        errors,
        vec![CustomDiagnostic::from_message(
            "Packages are limited to a single contract",
            FileId::default()
        )],
        "stdlib is producing warnings"
    );

    Ok(())
}

fn compile_contract_source(source: &str) -> noirc_artifacts::contract::CompiledContract {
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

    let (contract, _warnings) =
        noirc_driver::compile_contract(&mut context, root_crate_id, &CompileOptions::default())
            .expect("contract should compile successfully");
    contract
}

#[test]
fn abi_tag_collects_structs_and_globals() {
    let source = "
contract Foo {
    #[abi(foo)]
    pub global my_global: Field = 42;

    #[abi(bar)]
    pub struct MyStruct {
        inner: Field,
    }
}";

    let contract = compile_contract_source(source);

    // Check struct output
    let bar_structs = contract.outputs.structs.get("bar").expect("expected 'bar' tag in structs");
    assert_eq!(bar_structs.len(), 1);
    match &bar_structs[0] {
        AbiType::Struct { path, fields } => {
            assert!(path.contains("MyStruct"), "path should contain MyStruct, got: {path}");
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "inner");
            assert_eq!(fields[0].1, AbiType::Field);
        }
        other => panic!("expected AbiType::Struct, got: {other:?}"),
    }

    // Check global output
    let foo_globals = contract.outputs.globals.get("foo").expect("expected 'foo' tag in globals");
    assert_eq!(foo_globals.len(), 1);
    assert_eq!(foo_globals[0].name, "my_global");
    match &foo_globals[0].value {
        AbiValue::Integer { value, sign } => {
            assert!(!sign, "expected positive integer");
            assert_eq!(value, "000000000000000000000000000000000000000000000000000000000000002a");
        }
        other => panic!("expected AbiValue::Integer for Field = 42, got: {other:?}"),
    }
}

#[test]
fn abi_tag_collects_multiple_structs_under_same_tag() {
    let source = "
contract Foo {
    #[abi(things)]
    pub struct A {
        x: Field,
    }

    #[abi(things)]
    pub struct B {
        y: Field,
    }
}";

    let contract = compile_contract_source(source);

    let things = contract.outputs.structs.get("things").expect("expected 'things' tag in structs");
    assert_eq!(things.len(), 2);

    // Both should be structs; find them by path
    let mut found_a = false;
    let mut found_b = false;
    for abi_type in things {
        match abi_type {
            AbiType::Struct { path, fields } => {
                if path.contains("A") {
                    assert_eq!(fields.len(), 1);
                    assert_eq!(fields[0].0, "x");
                    assert_eq!(fields[0].1, AbiType::Field);
                    found_a = true;
                } else if path.contains("B") {
                    assert_eq!(fields.len(), 1);
                    assert_eq!(fields[0].0, "y");
                    assert_eq!(fields[0].1, AbiType::Field);
                    found_b = true;
                }
            }
            other => panic!("expected AbiType::Struct, got: {other:?}"),
        }
    }

    assert!(found_a, "struct A not found in 'things' tag");
    assert!(found_b, "struct B not found in 'things' tag");
}

#[test]
fn abi_global_with_non_literal_initializer_is_evaluated_at_comptime() {
    // Regression test for https://github.com/noir-lang/noir/issues/12695.
    // An `#[abi]` global initialized with a non-literal expression (here `2 + 3`) used
    // to ICE during ABI emission because the driver inspected the raw HIR expression
    // instead of the comptime-evaluated value.
    let source = "
contract Probe {
    #[abi(t)]
    pub global G: Field = 2 + 3;
}";

    let contract = compile_contract_source(source);

    let t_globals = contract.outputs.globals.get("t").expect("expected 't' tag in globals");
    assert_eq!(t_globals.len(), 1);
    match &t_globals[0].value {
        AbiValue::Integer { value, sign } => {
            assert!(!sign, "expected positive integer");
            assert_eq!(value, "0000000000000000000000000000000000000000000000000000000000000005");
        }
        other => panic!("expected AbiValue::Integer for Field = 5, got: {other:?}"),
    }
}

#[test]
fn abi_tag_preserves_global_names_under_same_tag() {
    let source = "
contract Foo {
    #[abi(constants)]
    pub global PURPOSE_AUTHORIZE: str<27> = \"Authorize Aztec Transaction\";

    #[abi(constants)]
    pub global FEE_AMOUNT: Field = 100;

    #[abi(error_codes)]
    pub global ERR_NOT_FOUND: Field = 1;

    #[abi(error_codes)]
    pub global ERR_FORBIDDEN: Field = 2;
}";

    let contract = compile_contract_source(source);

    let constants =
        contract.outputs.globals.get("constants").expect("expected 'constants' tag in globals");
    assert_eq!(constants.len(), 2);

    let purpose = constants
        .iter()
        .find(|g| g.name == "PURPOSE_AUTHORIZE")
        .expect("PURPOSE_AUTHORIZE not found");
    match &purpose.value {
        AbiValue::String { value } => assert_eq!(value, "Authorize Aztec Transaction"),
        other => panic!("expected AbiValue::String for PURPOSE_AUTHORIZE, got: {other:?}"),
    }

    let fee = constants.iter().find(|g| g.name == "FEE_AMOUNT").expect("FEE_AMOUNT not found");
    match &fee.value {
        AbiValue::Integer { value, sign } => {
            assert!(!sign);
            assert_eq!(value, "0000000000000000000000000000000000000000000000000000000000000064");
        }
        other => panic!("expected AbiValue::Integer for FEE_AMOUNT, got: {other:?}"),
    }

    let error_codes =
        contract.outputs.globals.get("error_codes").expect("expected 'error_codes' tag in globals");
    assert_eq!(error_codes.len(), 2);
    assert!(error_codes.iter().any(|g| g.name == "ERR_NOT_FOUND"));
    assert!(error_codes.iter().any(|g| g.name == "ERR_FORBIDDEN"));
}
