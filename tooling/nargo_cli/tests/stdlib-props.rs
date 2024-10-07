use std::{cell::RefCell, collections::BTreeMap, path::Path};

use acvm::{acir::native_types::WitnessStack, FieldElement};
use nargo::{
    ops::{execute_program, DefaultForeignCallExecutor},
    parse_all,
};
use noirc_abi::input_parser::InputValue;
use noirc_driver::{
    compile_main, file_manager_with_stdlib, prepare_crate, CompilationResult, CompileOptions,
    CompiledProgram,
};
use noirc_frontend::hir::Context;
use proptest::prelude::*;

/// Inputs and expected output of a snippet encoded in ABI format.
#[derive(Debug)]
struct SnippetInputOutput {
    inputs: BTreeMap<String, InputValue>,
    expected_output: InputValue,
}
impl SnippetInputOutput {
    fn new(inputs: Vec<(&str, InputValue)>, output: InputValue) -> Self {
        Self {
            inputs: inputs.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            expected_output: output,
        }
    }
}

/// Compile the main function in a code snippet.
fn compile_snippet(source: String) -> CompilationResult<CompiledProgram> {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );
    let parsed_files = parse_all(&file_manager);

    let mut context = Context::new(file_manager, parsed_files);
    let root_crate_id = prepare_crate(&mut context, file_name);

    compile_main(&mut context, root_crate_id, &CompileOptions::default(), None)
}

fn test_snippet(source: String, strategy: BoxedStrategy<SnippetInputOutput>) {
    let (program, _) = compile_snippet(source).expect("failed to compile");

    let blackbox_solver = bn254_blackbox_solver::Bn254BlackBoxSolver;
    let foreign_call_executor =
        RefCell::new(DefaultForeignCallExecutor::new(false, None, None, None));

    // Generate multiple input/output
    proptest!(|(io in strategy)| {
        let initial_witness = program.abi.encode(&io.inputs, None).expect("failed to encode");
        let mut foreign_call_executor = foreign_call_executor.borrow_mut();

        let witness_stack: WitnessStack<FieldElement> = execute_program(
            &program.program,
            initial_witness,
            &blackbox_solver,
            &mut *foreign_call_executor,
        )
        .expect("failed to execute");

        let main_witness = witness_stack.peek().expect("should have return value on witness stack");
        let main_witness = &main_witness.witness;

        let (_, return_value) = program.abi.decode(main_witness).expect("failed to decode");
        let return_value = return_value.expect("should decode a return value");

        prop_assert_eq!(return_value, io.expected_output)
    });
}

#[test]
fn works() {
    // TODO:
    // - define source to test, with a main function for entry
    // - create dummy create with the source code as in `prepare_source`
    // - compile using `create_main`
    // - put things into a RefCell so they can be reused
    // - define proptest! that encodes input and decodes output witness
    // - execute_program
    // - execute with interpreter
    // - execute as unconstrained because hashing functions detect it

    let program = "fn main(init: u32) -> pub u32 {
        let mut x = init;
        for i in 0 .. 6 {
            x += i;
        }
        x
    }";

    let strategy = any::<u32>()
        .prop_map(|init| {
            let init = init / 2;
            SnippetInputOutput::new(
                vec![("init", InputValue::Field(init.into()))],
                InputValue::Field((init + 15).into()),
            )
        })
        .boxed();

    test_snippet(program.to_string(), strategy);
}
