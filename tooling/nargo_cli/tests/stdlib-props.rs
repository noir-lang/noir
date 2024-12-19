use std::{cell::RefCell, collections::BTreeMap, path::Path};

use acvm::{acir::native_types::WitnessStack, AcirField, FieldElement};
use iter_extended::vecmap;
use nargo::{
    foreign_calls::DefaultForeignCallExecutor, ops::execute_program, parse_all, PrintOutput,
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
/// Use `force_brillig` to test it as an unconstrained function without having to change the code.
/// This is useful for methods that use the `runtime::is_unconstrained()` method to change their behavior.
fn prepare_and_compile_snippet(
    source: String,
    force_brillig: bool,
) -> CompilationResult<CompiledProgram> {
    let (mut context, root_crate_id) = prepare_snippet(source);
    let options = CompileOptions { force_brillig, ..Default::default() };
    // TODO: Run nargo::ops::transform_program?
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
        Err(e) => panic!("failed to compile program; brillig = {force_brillig}:\n{source}\n{e:?}"),
    };

    let blackbox_solver = bn254_blackbox_solver::Bn254BlackBoxSolver;
    let foreign_call_executor =
        RefCell::new(DefaultForeignCallExecutor::new(PrintOutput::None, None, None, None));

    // Generate multiple input/output
    proptest!(ProptestConfig::with_cases(100), |(io in strategy)| {
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

/// Run property tests on a code snippet which is assumed to execute a hashing function with the following signature:
///
/// ```ignore
/// fn main(input: [u8; {max_len}], message_size: u32) -> pub [u8; 32]
/// ```
///
/// The calls are executed with and without forcing brillig, because it seems common for hash functions to run different
/// code paths based on `runtime::is_unconstrained()`.
fn run_hash_proptest<const N: usize>(
    // Different generic maximum input sizes to try.
    max_lengths: &[usize],
    // Some hash functions allow inputs which are less than the generic parameters, others don't.
    variable_length: bool,
    // Make the source code specialized for a given expected input size.
    source: impl Fn(usize) -> String,
    // Rust implementation of the hash function.
    hash: fn(&[u8]) -> [u8; N],
) {
    for max_len in max_lengths {
        let max_len = *max_len;
        // The maximum length is used to pick the generic version of the method.
        let source = source(max_len);
        // Hash functions runs differently depending on whether the code is unconstrained or not.
        for force_brillig in [false, true] {
            let length_strategy =
                if variable_length { (0..=max_len).boxed() } else { Just(max_len).boxed() };
            // The actual input length can be up to the maximum.
            let strategy = length_strategy
                .prop_flat_map(|len| prop::collection::vec(any::<u8>(), len))
                .prop_map(move |mut msg| {
                    // The output is the hash of the data as it is.
                    let output = hash(&msg);

                    // The input has to be padded to the maximum length.
                    let msg_size = msg.len();
                    msg.resize(max_len, 0u8);

                    let mut inputs = vec![("input", bytes_input(&msg))];

                    // Omit the `message_size` if the hash function doesn't support it.
                    if variable_length {
                        inputs.push((
                            "message_size",
                            InputValue::Field(FieldElement::from(msg_size)),
                        ));
                    }

                    SnippetInputOutput::new(inputs, bytes_input(&output)).with_description(format!(
                        "force_brillig = {force_brillig}, max_len = {max_len}"
                    ))
                })
                .boxed();

            run_snippet_proptest(source.clone(), force_brillig, strategy);
        }
    }
}

/// This is just a simple test to check that property testing works.
#[test]
fn fuzz_basic() {
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

#[test]
fn fuzz_keccak256_equivalence() {
    run_hash_proptest(
        // XXX: Currently it fails with inputs >= 135 bytes
        &[0, 1, 100, 134],
        true,
        |max_len| {
            format!(
                "fn main(input: [u8; {max_len}], message_size: u32) -> pub [u8; 32] {{
                    std::hash::keccak256(input, message_size)
                }}"
            )
        },
        |data| sha3::Keccak256::digest(data).into(),
    );
}

#[test]
#[should_panic] // Remove once fixed
fn fuzz_keccak256_equivalence_over_135() {
    run_hash_proptest(
        &[135, 150],
        true,
        |max_len| {
            format!(
                "fn main(input: [u8; {max_len}], message_size: u32) -> pub [u8; 32] {{
                    std::hash::keccak256(input, message_size)
                }}"
            )
        },
        |data| sha3::Keccak256::digest(data).into(),
    );
}

#[test]
fn fuzz_sha256_equivalence() {
    run_hash_proptest(
        &[0, 1, 200, 511, 512],
        true,
        |max_len| {
            format!(
                "fn main(input: [u8; {max_len}], message_size: u64) -> pub [u8; 32] {{
                    std::hash::sha256_var(input, message_size)
                }}"
            )
        },
        |data| sha2::Sha256::digest(data).into(),
    );
}

#[test]
fn fuzz_sha512_equivalence() {
    run_hash_proptest(
        &[0, 1, 200],
        false,
        |max_len| {
            format!(
                "fn main(input: [u8; {max_len}]) -> pub [u8; 64] {{
                    std::hash::sha512::digest(input)
                }}"
            )
        },
        |data| sha2::Sha512::digest(data).into(),
    );
}

#[test]
fn fuzz_poseidon2_equivalence() {
    use bn254_blackbox_solver::poseidon_hash;

    // Test empty, small, then around the RATE value, then bigger inputs.
    for max_len in [0, 1, 3, 4, 100] {
        let source = format!(
            "fn main(input: [Field; {max_len}], message_size: u32) -> pub Field {{
                std::hash::poseidon2::Poseidon2::hash(input, message_size)
            }}"
        );

        let strategy = (0..=max_len)
            .prop_flat_map(field_vec_strategy)
            .prop_map(move |mut msg| {
                let output = poseidon_hash(&msg, msg.len() < max_len).expect("failed to hash");

                // The input has to be padded to the maximum length.
                let msg_size = msg.len();
                msg.resize(max_len, FieldElement::from(0u64));

                let inputs = vec![
                    ("input", InputValue::Vec(vecmap(msg, InputValue::Field))),
                    ("message_size", InputValue::Field(FieldElement::from(msg_size))),
                ];

                SnippetInputOutput::new(inputs, InputValue::Field(output))
                    .with_description(format!("max_len = {max_len}"))
            })
            .boxed();

        run_snippet_proptest(source.clone(), false, strategy);
    }
}

#[test]
fn fuzz_poseidon_equivalence() {
    use light_poseidon::{Poseidon, PoseidonHasher};

    let poseidon_hash = |inputs: &[FieldElement]| {
        let mut poseidon = Poseidon::<ark_bn254::Fr>::new_circom(inputs.len()).unwrap();
        let frs: Vec<ark_bn254::Fr> = inputs.iter().map(|f| f.into_repr()).collect::<Vec<_>>();
        let hash = poseidon.hash(&frs).expect("failed to hash");
        FieldElement::from_repr(hash)
    };

    // Noir has hashes up to length 16, but the reference library won't work with more than 12.
    for len in 1..light_poseidon::MAX_X5_LEN {
        let source = format!(
            "fn main(input: [Field; {len}]) -> pub Field {{
                let h1 = std::hash::poseidon::bn254::hash_{len}(input);
                let h2 = {{
                    let mut hasher = std::hash::poseidon::PoseidonHasher::default();
                    input.hash(&mut hasher);
                    hasher.finish()
                }};
                assert_eq(h1, h2);
                h1
            }}"
        );

        let strategy = field_vec_strategy(len)
            .prop_map(move |msg| {
                let output = poseidon_hash(&msg);
                let inputs = vec![("input", InputValue::Vec(vecmap(msg, InputValue::Field)))];

                SnippetInputOutput::new(inputs, InputValue::Field(output))
                    .with_description(format!("len = {len}"))
            })
            .boxed();

        run_snippet_proptest(source.clone(), false, strategy);
    }
}

fn bytes_input(bytes: &[u8]) -> InputValue {
    InputValue::Vec(
        bytes.iter().map(|b| InputValue::Field(FieldElement::from(*b as u32))).collect(),
    )
}

fn field_vec_strategy(len: usize) -> impl Strategy<Value = Vec<FieldElement>> {
    // Generate Field elements from random 32 byte vectors.
    let field = prop::collection::vec(any::<u8>(), 32)
        .prop_map(|bytes| FieldElement::from_be_bytes_reduce(&bytes));

    prop::collection::vec(field, len)
}
