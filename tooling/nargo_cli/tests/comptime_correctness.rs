mod common;

use std::sync::LazyLock;
use std::{cell::RefCell, collections::BTreeMap};

use acvm::{FieldElement, acir::native_types::WitnessStack};
use nargo::{foreign_calls::DefaultForeignCallBuilder, ops::execute_program};
use noirc_abi::input_parser::InputValue;
use proptest::prelude::*;

static NUM_CASES: LazyLock<u32> = LazyLock::new(|| {
    std::env::var("NOIR_COMPTIME_CORRECTNESS_TEST_NUM_CASES")
        .unwrap_or("1".into())
        .parse()
        .unwrap_or(1)
});

pub(crate) fn run_snippet(
    source: String,
    inputs: BTreeMap<String, InputValue>,
    force_brillig: bool,
) -> InputValue {
    let program = match common::prepare_and_compile_snippet(source.clone(), force_brillig) {
        Ok((program, _)) => program,
        Err(e) => panic!("failed to compile program; brillig = {force_brillig}:\n{source}\n{e:?}"),
    };

    let pedantic_solving = true;
    let blackbox_solver = bn254_blackbox_solver::Bn254BlackBoxSolver(pedantic_solving);
    let foreign_call_executor = RefCell::new(DefaultForeignCallBuilder::default().build());

    let initial_witness = program.abi.encode(&inputs, None).expect("failed to encode");
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
    return_value.expect("should decode a return value")
}

fn comptime_check_field_expression(
    strategy: BoxedStrategy<(String, &str, u32, u32)>,
    num_cases: u32,
    force_brillig: bool,
) {
    proptest!(ProptestConfig::with_cases(num_cases), |((comptime_expr, runtime_expr, a, b) in strategy)| {
        let program = format!("
        comptime fn comptime_code() -> Field {{
            {comptime_expr}
        }}

        fn runtime_code(a: Field, b: Field) -> Field {{
            {runtime_expr}
        }}

        fn main(a: Field, b: Field) -> pub Field {{
            assert_eq(comptime {{ comptime_code() }}, runtime_code(a, b));
            1
        }}");

        let inputs = vec![
            ("a", InputValue::Field(a.into())),
            ("b", InputValue::Field(b.into())),
        ];

        let inputs: BTreeMap<String, InputValue> = inputs.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

        let return_value = run_snippet(program.to_string(), inputs, force_brillig);
        prop_assert_eq!(return_value, InputValue::Field(1u32.into()));
    });
}

#[test]
fn comptime_check_field_add() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} + {b}"), "a + b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
fn comptime_check_field_add_brillig() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} + {b}"), "a + b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, true);
}

#[test]
fn comptime_check_field_sub() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} - {b}"), "a - b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
fn comptime_check_field_sub_brillig() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} - {b}"), "a - b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, true);
}

#[test]
fn comptime_check_field_div() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} / {b}"), "a / b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
fn comptime_check_field_div_brillig() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} / {b}"), "a / b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, true);
}

#[test]
fn comptime_check_field_mul() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} * {b}"), "a * b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
fn comptime_check_field_mul_brillig() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} * {b}"), "a * b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, true);
}

#[test]
#[ignore]
fn comptime_check_field_mod() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} % {b}"), "a % b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
#[ignore]
fn comptime_check_field_xor() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} ^ {b}"), "a ^ b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
#[ignore]
fn comptime_check_field_or() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} | {b}"), "a | b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
#[ignore]
fn comptime_check_field_and() {
    let strategy =
        any::<(u32, u32)>().prop_map(|(a, b)| (format!("{a} & {b}"), "a & b", a, b)).boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
#[ignore]
fn comptime_check_field_shl() {
    let strategy = any::<(u32, u8)>()
        .prop_map(|(a, b)| (format!("{a} << {b}"), "a << b", a, u32::from(b)))
        .boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}

#[test]
#[ignore]
fn comptime_check_field_shr() {
    let strategy = any::<(u32, u8)>()
        .prop_map(|(a, b)| (format!("{a} >> {b}"), "a >> b", a, u32::from(b)))
        .boxed();

    comptime_check_field_expression(strategy, *NUM_CASES, false);
}
