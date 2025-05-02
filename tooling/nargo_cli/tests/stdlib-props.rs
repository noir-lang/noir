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

/// The tests fuzz_zero_extent(), fuzz_signed_unsigned_same_size(), fuzz_sign_extent() and fuzz_truncate()
/// ensure that casting between integer types is correct, assuming casting to Field is correct.
/// Indeed, casting to Field is validated with fuzz_field_cast() test.
/// Any casting between integer types will use a combination of: no-op, zero extension, sign extension, or truncation.
/// Testing these 4 primitives should be enough to guarantee that casting between any integer types is correct.

/// Check that casting to Field is a no-op
#[test]
fn fuzz_field_cast() {
    let unsigned_8 = "fn main(x: u8) -> pub Field {
        x as Field
    }";
    let signed_8 = "fn main(x: i8) -> pub Field {
        x as Field
    }";
    let strategy_u8 = any::<u8>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field((x as u128).into()),
            )
        })
        .boxed();
    run_snippet_proptest(unsigned_8.to_string(), false, strategy_u8.clone());
    run_snippet_proptest(unsigned_8.to_string(), true, strategy_u8.clone());
    run_snippet_proptest(signed_8.to_string(), false, strategy_u8.clone());
    run_snippet_proptest(signed_8.to_string(), true, strategy_u8.clone());

    let unsigned_16 = "fn main(x: u16) -> pub Field {
        x as Field
    }";
    let signed_16 = "fn main(x: i16) -> pub Field {
        x as Field
    }";
    let strategy_u16 = any::<u16>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field((x as u128).into()),
            )
        })
        .boxed();
    run_snippet_proptest(unsigned_16.to_string(), false, strategy_u16.clone());
    run_snippet_proptest(unsigned_16.to_string(), true, strategy_u16.clone());
    run_snippet_proptest(signed_16.to_string(), false, strategy_u16.clone());
    run_snippet_proptest(signed_16.to_string(), true, strategy_u16.clone());

    let unsigned_32 = "fn main(x: u32) -> pub Field {
        x as Field
    }";
    let signed_32 = "fn main(x: i32) -> pub Field {
        x as Field
    }";
    let strategy_u32 = any::<u32>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field((x as u128).into()),
            )
        })
        .boxed();
    run_snippet_proptest(unsigned_32.to_string(), false, strategy_u32.clone());
    run_snippet_proptest(unsigned_32.to_string(), true, strategy_u32.clone());
    run_snippet_proptest(signed_32.to_string(), false, strategy_u32.clone());
    run_snippet_proptest(signed_32.to_string(), true, strategy_u32.clone());

    let unsigned_64 = "fn main(x: u64) -> pub Field {
        x as Field
    }";
    let signed_64 = "fn main(x: i64) -> pub Field {
        x as Field
    }";
    let strategy_u64 = any::<u64>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field((x as u128).into()),
            )
        })
        .boxed();
    run_snippet_proptest(unsigned_64.to_string(), false, strategy_u64.clone());
    run_snippet_proptest(unsigned_64.to_string(), true, strategy_u64.clone());
    run_snippet_proptest(signed_64.to_string(), false, strategy_u64.clone());
    run_snippet_proptest(signed_64.to_string(), true, strategy_u64.clone());

    let unsigned_128 = "fn main(x: u128) -> pub Field {
        x as Field
    }";

    let strategy_u128 = any::<u128>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(x.into()))],
                InputValue::Field(x.into()),
            )
        })
        .boxed();
    run_snippet_proptest(unsigned_128.to_string(), false, strategy_u128.clone());
    run_snippet_proptest(unsigned_128.to_string(), true, strategy_u128.clone());
}

/// Check that up-casting unsigned types is correct
#[test]
fn fuzz_zero_extent() {
    let unsigned_8_16 = "fn main(x: u8) -> pub Field {
    let y = x as u16;
    y as Field - x as Field
}";

    let unsigned_8_32 = "fn main(x: u8) -> pub Field {
    let y = x as u32;
    y as Field - x as Field
}";

    let unsigned_8_64 = "fn main(x: u8) -> pub Field {
    let y = x as u64;
    y as Field - x as Field
}";

    let unsigned_8_128 = "fn main(x: u8) -> pub Field {
    let y = x as u128;
    y as Field - x as Field
}";

    let unsigned_16_32 = "fn main(x: u16) -> pub Field {
    let y = x as u32;
    y as Field - x as Field
}";

    let unsigned_16_64 = "fn main(x: u16) -> pub Field {
    let y = x as u64;
    y as Field - x as Field
}";

    let unsigned_16_128 = "fn main(x: u16) -> pub Field {
    let y = x as u128;
    y as Field - x as Field
}";

    let unsigned_32_64 = "fn main(x: u32) -> pub Field {
    let y = x as u64;
    y as Field - x as Field
}";

    let unsigned_32_128 = "fn main(x: u32) -> pub Field {
    let y = x as u128;
    y as Field - x as Field
}";

    let unsigned_64_128 = "fn main(x: u64) -> pub Field {
    let y = x as u128;
    y as Field - x as Field
}";

    let unsigned_8_8 = "fn main(x: u8) -> pub Field {
    let y = x as u8;
    y as Field - x as Field
}";
    let unsigned_16_16 = "fn main(x: u16) -> pub Field {
    let y = x as u16;
    y as Field - x as Field
}";

    let unsigned_32_32 = "fn main(x: u32) -> pub Field {
    let y = x as u32;
    y as Field - x as Field
}";

    let unsigned_64_64 = "fn main(x: u64) -> pub Field {
    let y = x as u64;
    y as Field - x as Field
}";

    let unsigned_128_128 = "fn main(x: u128) -> pub Field {
    let y = x as u128;
   y as Field - x as Field
}";

    let strategy_u8 = any::<u8>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_u16 = any::<u16>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_u32 = any::<u32>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_u64 = any::<u64>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_u128 = any::<u128>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(x.into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();

    // zero extend 8 bits
    run_snippet_proptest(unsigned_8_8.to_string(), false, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_16.to_string(), false, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_32.to_string(), false, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_64.to_string(), false, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_128.to_string(), false, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_8.to_string(), true, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_16.to_string(), true, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_32.to_string(), true, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_64.to_string(), true, strategy_u8.clone());
    run_snippet_proptest(unsigned_8_128.to_string(), true, strategy_u8.clone());
    // zero extend 16 bits
    run_snippet_proptest(unsigned_16_16.to_string(), false, strategy_u16.clone());
    run_snippet_proptest(unsigned_16_16.to_string(), true, strategy_u16.clone());
    run_snippet_proptest(unsigned_16_32.to_string(), false, strategy_u16.clone());
    run_snippet_proptest(unsigned_16_32.to_string(), true, strategy_u16.clone());
    run_snippet_proptest(unsigned_16_64.to_string(), false, strategy_u16.clone());
    run_snippet_proptest(unsigned_16_64.to_string(), true, strategy_u16.clone());
    run_snippet_proptest(unsigned_16_128.to_string(), false, strategy_u16.clone());
    run_snippet_proptest(unsigned_16_128.to_string(), true, strategy_u16.clone());

    // zero extend 32 bits
    run_snippet_proptest(unsigned_32_32.to_string(), false, strategy_u32.clone());
    run_snippet_proptest(unsigned_32_32.to_string(), true, strategy_u32.clone());
    run_snippet_proptest(unsigned_32_64.to_string(), false, strategy_u32.clone());
    run_snippet_proptest(unsigned_32_64.to_string(), true, strategy_u32.clone());
    run_snippet_proptest(unsigned_32_128.to_string(), false, strategy_u32.clone());
    run_snippet_proptest(unsigned_32_128.to_string(), true, strategy_u32.clone());

    // zero extend 64 bits
    run_snippet_proptest(unsigned_64_64.to_string(), false, strategy_u64.clone());
    run_snippet_proptest(unsigned_64_64.to_string(), false, strategy_u64.clone());
    run_snippet_proptest(unsigned_64_128.to_string(), false, strategy_u64.clone());
    run_snippet_proptest(unsigned_64_128.to_string(), true, strategy_u64.clone());
    // zero extend 128 bits
    run_snippet_proptest(unsigned_128_128.to_string(), false, strategy_u128.clone());
    run_snippet_proptest(unsigned_128_128.to_string(), true, strategy_u128.clone());
}

/// Check that signed to unsigned and unsigned to signed, with the same bit size, do not change the inner value.
#[test]
fn fuzz_signed_unsigned_same_size() {
    let signed_unsigned_8_8 = "fn main(x: i8) -> pub Field {
    let y = x as u8;
    let z = y as i8;
    assert(z == x);
    y as Field - x as Field
}";
    let signed_unsigned_16_16 = "fn main(x: i16) -> pub Field {
    let y = x as u16;
    let z = y as i16;
    assert(z == x);
    y as Field - x as Field
}";

    let signed_unsigned_32_32 = "fn main(x: i32) -> pub Field {
    let y = x as u32;
    let z = y as i32;
    assert(z == x);
    y as Field - x as Field
}";

    let signed_unsigned_64_64 = "fn main(x: i64) -> pub Field {
    let y = x as u64;
    let z = y as i64;
    assert(z == x);
    y as Field - x as Field
}";

    let strategy_i8 = any::<u8>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_i16 = any::<u16>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_i32 = any::<u32>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_i64 = any::<u64>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();

    // zero extend 8 bits
    run_snippet_proptest(signed_unsigned_8_8.to_string(), false, strategy_i8.clone());
    run_snippet_proptest(signed_unsigned_8_8.to_string(), true, strategy_i8.clone());
    // zero extend 16 bits
    run_snippet_proptest(signed_unsigned_16_16.to_string(), false, strategy_i16.clone());
    run_snippet_proptest(signed_unsigned_16_16.to_string(), true, strategy_i16.clone());
    // zero extend 32 bits
    run_snippet_proptest(signed_unsigned_32_32.to_string(), false, strategy_i32.clone());
    run_snippet_proptest(signed_unsigned_32_32.to_string(), true, strategy_i32.clone());
    // zero extend 64 bits
    run_snippet_proptest(signed_unsigned_64_64.to_string(), false, strategy_i64.clone());
    run_snippet_proptest(signed_unsigned_64_64.to_string(), false, strategy_i64.clone());
}

#[test]
// Test sign extension
fn fuzz_sign_extent() {
    let signed_8_16 = "fn main(x: i8) -> pub Field {
     let neg = -x;
     let y = x as i16;
     let neg_y = neg as i16;
     (neg_y+y) as Field
    }";

    let signed_8_32 = "fn main(x: i8) -> pub Field {
     let neg = -x;
     let y = x as i32;
     let neg_y = neg as i32;
     (neg_y+y) as Field
    }";

    let signed_8_64 = "fn main(x: i8) -> pub Field {
     let neg = -x;
     let y = x as i64;
     let neg_y = neg as i64;
     (neg_y+y) as Field
    }";

    let signed_16_32 = "fn main(x: i16) -> pub Field {
     let neg = -x;
     let y = x as i32;
     let neg_y = neg as i32;
     (neg_y+y) as Field
    }";

    let signed_16_64 = "fn main(x: i16) -> pub Field {
     let neg = -x;
     let y = x as i64;
     let neg_y = neg as i64;
     (neg_y+y) as Field
    }";

    let signed_32_64 = "fn main(x: i32) -> pub Field {
     let neg = -x;
     let y = x as i64;
     let neg_y = neg as i64;
     (neg_y+y) as Field
    }";

    let strategy_i8 = any::<u8>()
        .prop_map(|mut x| {
            if x == 128 {
                x = 0;
            }
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();

    let strategy_i16 = any::<u16>()
        .prop_map(|mut x| {
            if x == 32768 {
                x = 0;
            }
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_i32 = any::<u32>()
        .prop_map(|mut x| {
            if x == 2147483648 {
                x = 0;
            }
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    // sign extent 8 bits
    run_snippet_proptest(signed_8_16.to_string(), false, strategy_i8.clone());
    run_snippet_proptest(signed_8_32.to_string(), false, strategy_i8.clone());
    run_snippet_proptest(signed_8_64.to_string(), false, strategy_i8.clone());
    run_snippet_proptest(signed_8_16.to_string(), true, strategy_i8.clone());
    run_snippet_proptest(signed_8_32.to_string(), true, strategy_i8.clone());
    run_snippet_proptest(signed_8_64.to_string(), true, strategy_i8.clone());
    // sign extent 16 bits
    run_snippet_proptest(signed_16_32.to_string(), false, strategy_i16.clone());
    run_snippet_proptest(signed_16_64.to_string(), false, strategy_i16.clone());
    run_snippet_proptest(signed_16_32.to_string(), true, strategy_i16.clone());
    run_snippet_proptest(signed_16_64.to_string(), true, strategy_i16.clone());
    // sign extent 32 bits
    run_snippet_proptest(signed_32_64.to_string(), false, strategy_i32.clone());
    run_snippet_proptest(signed_32_64.to_string(), true, strategy_i32.clone());
}

/// Check that truncation between unsigned types is correct
#[test]
fn fuzz_truncate() {
    let unsigned_16_8 = "fn main(x: u16) -> pub u16 {
        let y = x as u8;
        let q = (x as Field - y as Field)/256;
        (q as u16)*256 + y as u16
    }";
    let unsigned_32_8 = "fn main(x: u32) -> pub u32 {
        let y = x as u8;
        let q = (x as Field - y as Field)/256;
        (q as u32)*256 + y as u32
    }";
    let unsigned_32_16 = "fn main(x: u32) -> pub u32 {
        let y = x as u16;
        let q = (x as Field - y as Field)/65536;
        (q as u32)*65536 + y as u32
    }";

    let unsigned_64_8 = "fn main(x: u64) -> pub u64 {
        let y = x as u8;
        let q = (x as Field - y as Field)/256;
        (q as u64)*256 + y as u64
    }";
    let unsigned_64_16 = "fn main(x: u64) -> pub u64 {
        let y = x as u16;
        let q = (x as Field - y as Field)/65536;
        (q as u64)*65536 + y as u64
    }";
    let unsigned_64_32 = "fn main(x: u64) -> pub u64 {
        let y = x as u32;
        let q = (x as Field - y as Field)/4294967296;
        (q as u64)*4294967296 + y as u64
    }";
    let unsigned_128_8 = "fn main(x: u128) -> pub u128 {
        let y = x as u8;
        let q = (x as Field - y as Field)/256;
        (q as u128)*256 + y as u128
    }";
    let unsigned_128_16 = "fn main(x: u128) -> pub u128 {
        let y = x as u16;
        let q = (x as Field - y as Field)/65536;
        (q as u128)*65536 + y as u128
    }";
    let unsigned_128_32 = "fn main(x: u128) -> pub u128 {
        let y = x as u32;
        let q = (x as Field - y as Field)/4294967296;
        (q as u128)*4294967296 + y as u128
    }";
    let unsigned_128_64 = "fn main(x: u128) -> pub u128 {
        let y = x as u64;
        let q = (x as Field - y as Field)/18446744073709551616;
        (q as u128)*18446744073709551616 + y as u128
    }";

    let strategy_u16 = any::<u16>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field((x as u128).into()),
            )
        })
        .boxed();
    let strategy_u32 = any::<u32>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field((x as u128).into()),
            )
        })
        .boxed();
    let strategy_u64 = any::<u64>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field((x as u128).into()))],
                InputValue::Field((x as u128).into()),
            )
        })
        .boxed();
    let strategy_u128 = any::<u128>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(x.into()))],
                InputValue::Field(x.into()),
            )
        })
        .boxed();

    run_snippet_proptest(unsigned_16_8.to_string(), false, strategy_u16.clone());
    run_snippet_proptest(unsigned_16_8.to_string(), true, strategy_u16.clone());
    run_snippet_proptest(unsigned_32_8.to_string(), false, strategy_u32.clone());
    run_snippet_proptest(unsigned_32_8.to_string(), true, strategy_u32.clone());
    run_snippet_proptest(unsigned_32_16.to_string(), false, strategy_u32.clone());
    run_snippet_proptest(unsigned_32_16.to_string(), true, strategy_u32.clone());
    run_snippet_proptest(unsigned_64_8.to_string(), false, strategy_u64.clone());
    run_snippet_proptest(unsigned_64_8.to_string(), true, strategy_u64.clone());
    run_snippet_proptest(unsigned_64_16.to_string(), false, strategy_u64.clone());
    run_snippet_proptest(unsigned_64_16.to_string(), true, strategy_u64.clone());
    run_snippet_proptest(unsigned_64_32.to_string(), false, strategy_u64.clone());
    run_snippet_proptest(unsigned_64_32.to_string(), true, strategy_u64.clone());
    run_snippet_proptest(unsigned_128_8.to_string(), false, strategy_u128.clone());
    run_snippet_proptest(unsigned_128_8.to_string(), true, strategy_u128.clone());
    run_snippet_proptest(unsigned_128_16.to_string(), false, strategy_u128.clone());
    run_snippet_proptest(unsigned_128_16.to_string(), true, strategy_u128.clone());
    run_snippet_proptest(unsigned_128_32.to_string(), false, strategy_u128.clone());
    run_snippet_proptest(unsigned_128_32.to_string(), true, strategy_u128.clone());
    run_snippet_proptest(unsigned_128_64.to_string(), false, strategy_u128.clone());
    run_snippet_proptest(unsigned_128_64.to_string(), true, strategy_u128.clone());
}
