use std::{cell::RefCell, collections::BTreeMap, path::Path};

use acvm::{acir::native_types::WitnessStack, FieldElement};
use nargo::{
    ops::{execute_program, DefaultForeignCallExecutor},
    parse_all,
};
use noirc_abi::input_parser::InputValue;
use noirc_driver::{
    compile_main, file_manager_with_stdlib, prepare_crate, CompilationResult, CompileOptions,
    CompiledProgram, CrateId,
};
use noirc_frontend::hir::Context;
use proptest::prelude::*;
use sha3::Digest;

/// Inputs and expected output of a snippet encoded in ABI format.
#[derive(Debug)]
struct SnippetInputOutput {
    description: String,
    inputs: BTreeMap<String, InputValue>,
    expected_output: InputValue,
}
impl SnippetInputOutput {
    fn new(inputs: Vec<(&str, InputValue)>, output: InputValue) -> Self {
        Self {
            description: "".to_string(),
            inputs: inputs.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            expected_output: output,
        }
    }

    /// Attach some description to hint at the scenario we are testing.
    fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
}

/// Prepare a code snippet.
fn prepare_snippet(source: String) -> (Context<'static, 'static>, CrateId) {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(file_name, source).expect(
        "Adding source buffer to file manager should never fail when file manager is empty",
    );
    let parsed_files = parse_all(&file_manager);

    let mut context = Context::new(file_manager, parsed_files);
    let root_crate_id = prepare_crate(&mut context, file_name);

    (context, root_crate_id)
}

/// Compile the main function in a code snippet.
///
/// Use `force_brillig` to test it as an unconstrainted function without having to change the code.
/// This is useful for methods that use the `runtime::is_uncontrained()` method to change their behaviour.
fn prepare_and_compile_snippet(
    source: String,
    force_brillig: bool,
) -> CompilationResult<CompiledProgram> {
    let (mut context, root_crate_id) = prepare_snippet(source);
    let options = CompileOptions { force_brillig, ..Default::default() };
    compile_main(&mut context, root_crate_id, &options, None)
}

/// Compile a snippet and run property tests against it by generating random input/output pairs
/// according to the strategy, executing the snippet with the input, and asserting that the
/// output it returns is the one we expect.
fn run_snippet_proptest(
    source: String,
    force_brillig: bool,
    strategy: BoxedStrategy<SnippetInputOutput>,
) {
    let program = match prepare_and_compile_snippet(source.clone(), force_brillig) {
        Ok((program, _)) => program,
        Err(e) => panic!("failed to compile program:\n{source}\n{e:?}"),
    };

    let blackbox_solver = bn254_blackbox_solver::Bn254BlackBoxSolver;
    let foreign_call_executor =
        RefCell::new(DefaultForeignCallExecutor::new(false, None, None, None));

    // Generate multiple input/output
    // TODO: Execute with the interpreter as well.
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

        prop_assert_eq!(return_value, io.expected_output, "{}", io.description);
    });
}

/// This is just a simple test to check that property testing works.
#[test]
fn test_basic() {
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

    run_snippet_proptest(program.to_string(), false, strategy);
}

// TODO: - Sha256, Keccak256, Sha512, Schnorr, Poseidon2 and Poseidon

#[test]
fn test_keccak256() {
    fn hash(input: &[u8]) -> [u8; 32] {
        sha3::Keccak256::digest(input).try_into().expect("result is 256 bits")
    }

    // Keccak256 runs differently depending on whether it's unconstrained or not.
    for force_brillig in [false, true] {
        // XXX: Currently it fails with inputs >= 135 bytes
        for max_len in [0usize, 10, 100, 134] {
            // The maximum length is used to pick the generic version of the method.
            let program = format!(
                "fn main(input: [u8; {max_len}], message_size: u32) -> pub [u8; 32] {{
                    std::hash::keccak256(input, message_size)
                }}"
            );

            // The actual input length can be up to the maximum.
            let strategy = (0..=max_len)
                .prop_flat_map(|len| prop::collection::vec(any::<u8>(), len))
                .prop_map(move |mut msg| {
                    // The output is the hash of the data as it is.
                    let output = hash(&msg);

                    // The input has to be padded to the maximum length.
                    let msg_size = msg.len();
                    msg.resize(max_len, 0u8);

                    SnippetInputOutput::new(
                        vec![
                            ("input", bytes_input(&msg)),
                            ("message_size", InputValue::Field(FieldElement::from(msg_size))),
                        ],
                        bytes_input(&output),
                    )
                    .with_description(format!(
                        "force_brillig = {force_brillig}, max_len = {max_len}"
                    ))
                })
                .boxed();

            run_snippet_proptest(program.to_string(), force_brillig, strategy);
        }
    }
}

fn bytes_input(bytes: &[u8]) -> InputValue {
    InputValue::Vec(
        bytes.iter().map(|b| InputValue::Field(FieldElement::from(*b as u32))).collect(),
    )
}
