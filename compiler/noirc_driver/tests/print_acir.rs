//! Integration tests for the `--print-acir` display path.
//!
//! These tests compile a Noir program end-to-end and snapshot the formatted
//! ACIR/Brillig that `nargo compile --print-acir` would print, so we can lock
//! in the rendering of static assertion payloads alongside the opcodes that
//! check them.

use std::path::Path;

use acvm::acir::circuit::Circuit;
use acvm::acir::native_types::Witness;
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
fn print_acir_renders_brillig_assertion_payload() {
    let source = r#"
    fn main(x: u32) {
        assert_eq(x, 0, "x is not zero");
    }
    "#;

    let program = compile(source, true);
    let displayed = display_compiled_program(&program);

    // Locate the IndirectConst that loads the error selector and check it has
    // a `// "x is not zero"` annotation. We don't snapshot the entire Brillig
    // bytecode because the surrounding opcodes are sensitive to unrelated
    // codegen changes; the annotation is the bit this PR is locking in.
    let annotated_line = displayed
        .lines()
        .find(|line| line.contains("indirect const") && line.contains("// \"x is not zero\""))
        .unwrap_or_else(|| panic!("expected an annotated indirect const line, got:\n{displayed}"));

    insta::assert_snapshot!(annotated_line, @r#"17: @1 = indirect const u64 1591142006424964070 // "x is not zero""#);
}
