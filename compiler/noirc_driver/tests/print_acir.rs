//! Integration tests for the `--print-acir` display path.
//!
//! These tests compile a Noir program end-to-end and snapshot the formatted
//! ACIR/Brillig that `nargo compile --print-acir` would print, so we can lock
//! in the rendering of static assertion payloads alongside the opcodes that
//! check them.

use std::path::Path;

use acvm::acir::circuit::Circuit;
use acvm::acir::native_types::Witness;
use noirc_abi::AbiErrorType;
use noirc_driver::{
    CompileOptions, display_compiled_program, file_manager_with_stdlib, prepare_crate,
};
use noirc_frontend::hir::{Context, def_map::parse_file};

fn compile(source: &str, force_brillig: bool) -> noirc_artifacts::program::CompiledProgram {
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

    let options = CompileOptions { force_brillig, ..Default::default() };
    let (program, _warnings) =
        noirc_driver::compile_main(&mut context, root_crate_id, &options, None)
            .expect("program should compile successfully");
    program
}

#[test]
fn print_acir_renders_static_assertion_payload() {
    let source = r#"
    fn main(x: u32) {
        assert_eq(x, 0, "x is not zero");
    }
    "#;

    let program = compile(source, false);
    let displayed = display_compiled_program(&program);

    insta::assert_snapshot!(displayed, @r"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: []
    ASSERT w0 = 0 // x is not zero
    ");

    // The displayed ACIR should round-trip through the parser: the trailing
    // `// message` is treated as a comment, so the parsed circuit's opcodes
    // match the original.
    let circuit_text = displayed
        .strip_prefix("func 0\n")
        .expect("displayed program should start with the `func 0` header");
    let parsed = Circuit::from_str(circuit_text).expect("ACIR display should be parseable");
    assert_eq!(parsed.private_parameters, [Witness(0)].into_iter().collect());
    assert_eq!(parsed.opcodes, program.program.functions[0].opcodes);
}

#[test]
fn folded_generic_str_assertion_has_no_stale_error_type() {
    // A generic `str<N>` assertion message records a dynamic `custom string` error
    // selector during SSA generation, but the constant string folds to a static-string
    // assertion payload during ACIR lowering. The ABI should only advertise the
    // reachable static-string selector, not the stale pre-fold dynamic selector.
    let source = r#"
    fn fail_with_generic_msg<T>(predicate: bool, msg: T) {
        assert(predicate, msg);
    }

    fn main(x: pub Field) {
        fail_with_generic_msg(x == 0, "bad");
    }
    "#;

    let program = compile(source, false);

    let error_types: Vec<&AbiErrorType> = program.abi.error_types.values().collect();
    assert_eq!(
        error_types,
        vec![&AbiErrorType::String { string: "bad".to_string() }],
        "expected only the reachable static-string error type"
    );
}

#[test]
fn dynamic_custom_error_type_is_preserved() {
    // A non-string assertion payload is never folded to a static string, so its dynamic
    // selector is genuinely emitted and must remain in the ABI.
    let source = r#"
    struct MyError {
        code: Field,
    }

    fn main(x: pub Field) {
        assert(x == 0, MyError { code: x });
    }
    "#;

    let program = compile(source, false);

    let error_types: Vec<&AbiErrorType> = program.abi.error_types.values().collect();
    assert_eq!(error_types.len(), 1, "expected the dynamic custom error type to be retained");
    assert!(
        matches!(error_types[0], AbiErrorType::Custom(_)),
        "expected a custom error type, got {:?}",
        error_types[0]
    );
}

#[test]
fn print_acir_renders_brillig_assertion_payload() {
    let source = r#"
    fn main(x: u32) {
        assert_eq(x, 0, "x is not zero");
    }
    "#;

    let program = compile(source, true);
    let displayed = display_compiled_program(&program);

    insta::assert_snapshot!(displayed, @r#"
    func 0
    private parameters: [w0]
    public parameters: []
    return values: []
    BRILLIG CALL func: 0, predicate: 1, inputs: [w0], outputs: []

    unconstrained func 0: main
     0: @2 = const u32 1
     1: @1 = const u32 32836
     2: @0 = const u32 68
     3: sp[3] = const u32 1
     4: sp[4] = const u32 0
     5: @67 = calldata copy [sp[4]; sp[3]]
     6: @67 = cast @67 to u32
     7: sp[2] = @67
     8: call 12
     9: sp[2] = const u32 68
    10: sp[3] = const u32 0
    11: stop @[sp[2]; sp[3]]
    12: sp[3] = const u32 0
    13: sp[4] = u32 eq sp[2], sp[3]
    14: jump if sp[4] to 16
    15: call 17
    16: return
    17: @1 = indirect const u64 1591142006424964070 // "x is not zero"
    18: trap @[@1; @2]
    19: return
    "#);
}
