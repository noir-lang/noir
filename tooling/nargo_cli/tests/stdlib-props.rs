mod common;

use std::cell::RefCell;
use std::collections::BTreeMap;

use acvm::{AcirField, FieldElement, acir::native_types::WitnessStack};
use iter_extended::vecmap;
use nargo::{foreign_calls::DefaultForeignCallBuilder, ops::execute_program};
use noirc_abi::input_parser::InputValue;
use proptest::prelude::*;

/// Inputs and expected output of a snippet encoded in ABI format.
#[derive(Debug)]
struct SnippetInputOutput {
    pub description: String,
    pub inputs: BTreeMap<String, InputValue>,
    pub expected_output: InputValue,
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

/// Compile a snippet and run property tests against it by generating random input/output pairs
/// according to the strategy, executing the snippet with the input, and asserting that the
/// output it returns is the one we expect.
fn run_snippet_proptest(
    source: String,
    force_brillig: bool,
    strategy: BoxedStrategy<SnippetInputOutput>,
) {
    let program = match common::prepare_and_compile_snippet(source.clone(), force_brillig) {
        Ok((program, _)) => program,
        Err(e) => panic!("failed to compile program; brillig = {force_brillig}:\n{source}\n{e:?}"),
    };

    let pedantic_solving = true;
    let blackbox_solver = bn254_blackbox_solver::Bn254BlackBoxSolver(pedantic_solving);
    let foreign_call_executor = RefCell::new(DefaultForeignCallBuilder::default().build());

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

fn field_vec_strategy(len: usize) -> impl Strategy<Value = Vec<FieldElement>> {
    // Generate Field elements from random 32 byte vectors.
    let field = prop::collection::vec(any::<u8>(), 32)
        .prop_map(|bytes| FieldElement::from_be_bytes_reduce(&bytes));

    prop::collection::vec(field, len)
}
