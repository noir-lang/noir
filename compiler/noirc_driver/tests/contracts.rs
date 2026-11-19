use std::path::Path;

use fm::FileId;
use noirc_abi::{AbiType, AbiValue};
use noirc_driver::{CompileOptions, ErrorsAndWarnings, file_manager_with_stdlib, prepare_crate};
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::{Context, ParsedFiles, def_map::parse_file};

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
    let parsed_files: ParsedFiles = file_manager
        .as_file_map()
        .all_file_ids()
        .map(|&file_id| (file_id, parse_file(&file_manager, file_id)))
        .collect();

    let mut context = Context::new(file_manager, parsed_files);
    let root_crate_id = prepare_crate(&mut context, file_name);

    let errors = noirc_driver::compile_contract(
        context,
        root_crate_id,
        &CompileOptions::default(),
        Path::new(""),
    )
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

fn compile_contract_with_warnings(
    source: &str,
) -> (noirc_artifacts::contract::CompiledContract, Vec<CustomDiagnostic>) {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source.to_owned()).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );
    let parsed_files: ParsedFiles = file_manager
        .as_file_map()
        .all_file_ids()
        .map(|&file_id| (file_id, parse_file(&file_manager, file_id)))
        .collect();

    let mut context = Context::new(file_manager, parsed_files);
    let root_crate_id = prepare_crate(&mut context, file_name);

    noirc_driver::compile_contract(
        context,
        root_crate_id,
        &CompileOptions::default(),
        Path::new(""),
    )
    .expect("contract should compile successfully")
}

fn compile_contract_source(source: &str) -> noirc_artifacts::contract::CompiledContract {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source.to_owned()).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );
    let parsed_files: ParsedFiles = file_manager
        .as_file_map()
        .all_file_ids()
        .map(|&file_id| (file_id, parse_file(&file_manager, file_id)))
        .collect();

    let mut context = Context::new(file_manager, parsed_files);
    let root_crate_id = prepare_crate(&mut context, file_name);

    let (contract, _warnings) = noirc_driver::compile_contract(
        context,
        root_crate_id,
        &CompileOptions::default(),
        Path::new(""),
    )
    .expect("contract should compile successfully");
    contract
}

#[test]
fn reports_always_false_assertion_for_contracts() {
    let source = "
fn check<let CONSTRAINED: u32, let HANDSHAKE: u32>(x: Field) -> Field {
    if CONSTRAINED == 1 {
        assert(HANDSHAKE == 1, \"constrained delivery requires a handshake origin\");
    }
    x
}

contract VanillaContract {
    pub fn entrypoint(x: pub Field) -> pub Field {
        crate::check::<1, 0>(x)
    }
}";

    let (_contract, warnings) = compile_contract_with_warnings(source);

    assert!(
        warnings.iter().any(|warning| warning.message.contains("Assertion is always false")),
        "expected an 'Assertion is always false' diagnostic, got: {warnings:?}"
    );
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
fn abi_global_struct_value_uses_declared_field_order() {
    // Regression test for https://github.com/noir-lang/noir-claude/issues/1375.
    // A struct global written with constructor fields out of declaration order used to
    // emit `AbiValue::Struct.fields` in source-constructor order instead of the struct's
    // declared field order, leaving downstream consumers that index fields by position
    // with semantically misplaced values.
    let source = "
contract Foo {
    pub struct Pair {
        first: Field,
        second: Field,
    }

    #[abi(g)]
    pub global REVERSED: Pair = Pair { second: 2, first: 1 };

    #[abi(g)]
    pub global NORMAL: Pair = Pair { first: 1, second: 2 };
}";

    let contract = compile_contract_source(source);

    let globals = contract.outputs.globals.get("g").expect("expected 'g' tag in globals");

    let expected = [
        ("first", "0000000000000000000000000000000000000000000000000000000000000001"),
        ("second", "0000000000000000000000000000000000000000000000000000000000000002"),
    ];

    // Both globals are semantically `Pair { first: 1, second: 2 }` and must serialize
    // with the same declared field order regardless of how the constructor was written.
    for name in ["REVERSED", "NORMAL"] {
        let global = globals.iter().find(|g| g.name == name).expect("global not found");
        match &global.value {
            AbiValue::Struct { fields } => {
                assert_eq!(fields.len(), 2, "{name} should have two fields");
                for ((field_name, field_value), (expected_name, expected_value)) in
                    fields.iter().zip(expected)
                {
                    assert_eq!(field_name, expected_name, "{name} field name out of order");
                    match field_value {
                        AbiValue::Integer { value, sign } => {
                            assert!(!sign, "{name}.{field_name} should be positive");
                            assert_eq!(value, expected_value, "{name}.{field_name} value mismatch");
                        }
                        other => panic!("expected AbiValue::Integer, got: {other:?}"),
                    }
                }
            }
            other => panic!("expected AbiValue::Struct for {name}, got: {other:?}"),
        }
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

/// Compiles `source` on a rayon pool of `threads` threads, so its entry points are compiled one
/// at a time with one thread and in parallel with more.
fn compile_contract_on_threads(
    source: &str,
    threads: usize,
) -> (noirc_artifacts::contract::CompiledContract, Vec<CustomDiagnostic>) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(|| compile_contract_with_warnings(source))
}

#[test]
fn compiling_entry_points_in_parallel_gives_the_sequential_result() {
    // Each entry point warns with a message of its own, so the warnings' order shows the order
    // the entry points' results were collected in.
    let mut source = String::from(
        "
fn check<let CONSTRAINED: u32, let HANDSHAKE: u32>(x: Field, message: str<2>) -> Field {
    if CONSTRAINED == 1 {
        assert(HANDSHAKE == 1, message);
    }
    x
}

contract Foo {
",
    );
    let names: Vec<String> = (0..8).map(|i| format!("f{i}")).collect();
    for (i, name) in names.iter().enumerate() {
        source.push_str(&format!(
            "    pub fn {name}(x: pub Field) -> pub Field {{ crate::check::<1, 0>(x + {i}, \"m{i}\") }}\n"
        ));
    }
    source.push('}');

    let (sequential, sequential_warnings) = compile_contract_on_threads(&source, 1);
    let (parallel, parallel_warnings) = compile_contract_on_threads(&source, 4);

    let function_names: Vec<&str> =
        sequential.functions.iter().map(|function| function.name.as_str()).collect();
    assert_eq!(function_names, names, "functions should be in declaration order");
    assert!(sequential_warnings.len() >= names.len(), "expected a warning per entry point");

    assert_eq!(
        serde_json::to_value(&parallel.functions).unwrap(),
        serde_json::to_value(&sequential.functions).unwrap(),
        "parallel compilation should produce the same functions"
    );
    let messages = |warnings: &[CustomDiagnostic]| -> Vec<String> {
        warnings
            .iter()
            .map(|warning| format!("{} {:?}", warning.message, warning.secondaries))
            .collect()
    };
    assert_eq!(
        messages(&parallel_warnings),
        messages(&sequential_warnings),
        "parallel compilation should report the same warnings in the same order"
    );
}
