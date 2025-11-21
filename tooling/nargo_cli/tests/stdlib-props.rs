mod common;

use std::cell::RefCell;
use std::collections::BTreeMap;

use acvm::{FieldElement, acir::native_types::WitnessStack, brillig_vm};
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
            brillig_vm::Version::default()
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

fn make_field_cast_test(size: u32, signed: bool) -> String {
    if signed {
        format!(
            "fn main(x: i{size}) -> pub Field {{
            x as u{size} as Field
         }}
         "
        )
    } else {
        format!(
            "fn main(x: u{size}) -> pub Field {{
            x as Field
}}"
        )
    }
}

fn get_unsigned_strategies() -> Vec<(u32, BoxedStrategy<SnippetInputOutput>)> {
    let strategy_u8 = any::<u8>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(u128::from(x).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_u16 = any::<u16>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(u128::from(x).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_u32 = any::<u32>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(u128::from(x).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    let strategy_u64 = any::<u64>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(u128::from(x).into()))],
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
    vec![
        (8, strategy_u8),
        (16, strategy_u16),
        (32, strategy_u32),
        (64, strategy_u64),
        (128, strategy_u128),
    ]
}
fn get_signed_strategies() -> Vec<(u32, BoxedStrategy<SnippetInputOutput>)> {
    let strategy_i8 = any::<u8>()
        .prop_map(|mut x| {
            if x == 128 {
                x = 0;
            }
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(u128::from(x).into()))],
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
                vec![("x", InputValue::Field(u128::from(x).into()))],
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
                vec![("x", InputValue::Field(u128::from(x).into()))],
                InputValue::Field(0_u128.into()),
            )
        })
        .boxed();
    vec![(8, strategy_i8), (16, strategy_i16), (32, strategy_i32)]
}

fn get_truncate_strategies() -> Vec<(u32, BoxedStrategy<SnippetInputOutput>)> {
    let strategy_u16 = any::<u16>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(u128::from(x).into()))],
                InputValue::Field(u128::from(x).into()),
            )
        })
        .boxed();
    let strategy_u32 = any::<u32>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(u128::from(x).into()))],
                InputValue::Field(u128::from(x).into()),
            )
        })
        .boxed();
    let strategy_u64 = any::<u64>()
        .prop_map(|x| {
            SnippetInputOutput::new(
                vec![("x", InputValue::Field(u128::from(x).into()))],
                InputValue::Field(u128::from(x).into()),
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

    vec![(16, strategy_u16), (32, strategy_u32), (64, strategy_u64), (128, strategy_u128)]
}

/// The tests fuzz_zero_extend(), fuzz_signed_unsigned_same_size(), fuzz_sign_extend() and fuzz_truncate()
/// ensure that casting between integer types is correct, assuming casting to Field is correct.
/// Casting to Field is validated with the fuzz_field_cast() test.
/// Any casting between integer types will use a combination of: no-op, zero extension, sign extension, or truncation.
/// Testing these 4 primitives should be enough to guarantee that casting between any integer types is correct.
///
/// Check that casting to Field is a no-op
#[test]
fn fuzz_field_cast() {
    for (size, strategy) in get_truncate_strategies().iter() {
        if *size < 128 {
            let signed_i = make_field_cast_test(*size, true);
            run_snippet_proptest(signed_i.clone(), false, strategy.clone());
            run_snippet_proptest(signed_i, true, strategy.clone());
        }
        let unsigned_i = make_field_cast_test(*size, false);
        run_snippet_proptest(unsigned_i.clone(), false, strategy.clone());
        run_snippet_proptest(unsigned_i, true, strategy.clone());
    }
}

fn make_zero_extend_test(in_size: u32, out_size: u32) -> String {
    format!(
        "fn main(x: u{in_size}) -> pub Field {{
        let y = x as u{out_size};
        y as Field - x as Field
}}"
    )
}

/// Check that up-casting unsigned types is correct
#[test]
fn fuzz_zero_extend() {
    let strategies = get_unsigned_strategies();
    // zero extend 8, 16, 32, 64, 128 bits
    for (size, strategy) in strategies.iter() {
        for i in [8, 16, 32, 64, 128].iter() {
            if *i >= *size {
                let unsigned_j_i = make_zero_extend_test(*size, *i);
                run_snippet_proptest(unsigned_j_i.clone(), false, strategy.clone());
                run_snippet_proptest(unsigned_j_i, true, strategy.clone());
            }
        }
    }
}

fn make_sign_unsigned_test(size: u32) -> String {
    format!(
        "fn main(x: i{size}) -> pub Field {{
    let y = x as u{size};
    let z = y as i{size};
    assert(z == x);
    0
}}"
    )
}

/// Check that signed to unsigned and unsigned to signed, with the same bit size, do not change the inner value.
#[test]
fn fuzz_signed_unsigned_same_size() {
    let strategies = get_unsigned_strategies();

    for (size, strategy) in strategies.iter() {
        if *size < 128 {
            let signed_unsigned_i_i = make_sign_unsigned_test(*size);
            run_snippet_proptest(signed_unsigned_i_i.clone(), false, strategy.clone());
            run_snippet_proptest(signed_unsigned_i_i, true, strategy.clone());
        }
    }
}

fn make_sign_extend_test(in_size: u32, out_size: u32) -> String {
    format!(
        "fn main(x: i{in_size}) -> pub Field {{
         let neg = -x;
     let y = x as i{out_size};
     let neg_y = neg as i{out_size};
     (neg_y+y) as u{out_size} as Field
}}"
    )
}
#[test]
// Test sign extension
fn fuzz_sign_extend() {
    for (size, strategy) in get_signed_strategies().iter() {
        for i in [16, 32, 64] {
            if i > *size {
                // sign extend
                let signed_i_i = make_sign_extend_test(*size, i);
                run_snippet_proptest(signed_i_i.clone(), false, strategy.clone());
                run_snippet_proptest(signed_i_i, true, strategy.clone());
            }
        }
    }
}

fn make_truncate_test(size: u32, truncate: u32) -> String {
    let max: u128 = 1 << truncate;

    format!(
        "fn main(x: u{size}) -> pub u{size} {{
        let y = x as u{truncate};
        let q = (x as Field - y as Field)/{max};
        (q as u{size})*{max} + y as u{size}
}}"
    )
}
/// Check that truncation between unsigned types is correct
#[test]
fn fuzz_truncate() {
    for (size, strategy) in get_truncate_strategies().iter() {
        for i in [8, 16, 32, 64] {
            if i < *size {
                let unsigned_j_i = make_truncate_test(*size, i);
                run_snippet_proptest(unsigned_j_i.clone(), false, strategy.clone());
                run_snippet_proptest(unsigned_j_i, true, strategy.clone());
            }
        }
    }
}
