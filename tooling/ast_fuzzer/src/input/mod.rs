use acvm::{AcirField, FieldElement};
use num_bigint::BigInt;
use arbitrary::Unstructured;

use dictionary::build_dictionary_from_ssa;
use noir_greybox_fuzzer::build_dictionary_from_program;
use noirc_abi::{Abi, AbiType, InputMap, Sign, input_parser::InputValue};
use noirc_evaluator::ssa::ssa_gen::Ssa;
use proptest::{
    prelude::*,
    test_runner::{Config, RngAlgorithm, TestRng, TestRunner},
};
use std::collections::{BTreeMap, BTreeSet};

mod dictionary;
mod int;
mod uint;

use int::IntStrategy;
use uint::UintStrategy;

/// Generate arbitrary inputs for a compiled program according to the ABI.
pub fn arb_inputs(
    u: &mut Unstructured,
    program: &acir::circuit::Program<FieldElement>,
    abi: &Abi,
) -> arbitrary::Result<InputMap> {
    // Reuse the proptest strategy in `noir_fuzzer` to generate random inputs.
    let dictionary = build_dictionary_from_program(program);
    let dictionary = BTreeSet::from_iter(dictionary);
    let strategy = arb_input_map(abi, &dictionary);
    arb_value_tree(u, strategy)
}

/// Generate arbitrary inputs for an SSA of a program, according to the ABI.
pub(crate) fn arb_inputs_from_ssa(
    u: &mut Unstructured,
    ssa: &Ssa,
    abi: &Abi,
) -> arbitrary::Result<InputMap> {
    // Reuse the proptest strategy in `noir_fuzzer` to generate random inputs.
    let dictionary = build_dictionary_from_ssa(ssa);
    let strategy = arb_input_map(abi, &dictionary);
    arb_value_tree(u, strategy)
}

/// Generate a seed and use it to generate an arbitrary value using the proptest strategy.
fn arb_value_tree(
    u: &mut Unstructured,
    strategy: BoxedStrategy<InputMap>,
) -> arbitrary::Result<InputMap> {
    // The strategy needs a runner, although all it really uses is the RNG from it.
    let seed: [u8; 16] = u.arbitrary()?;
    let rng = TestRng::from_seed(RngAlgorithm::XorShift, &seed);
    let mut runner = TestRunner::new_with_rng(Config::default(), rng);
    let tree = strategy.new_tree(&mut runner).map_err(|_| arbitrary::Error::IncorrectFormat)?;
    Ok(tree.current())
}

/// Given the [Abi] description of a Noir program, generate random [InputValue]s for each circuit parameter.
///
/// Use the `dictionary` to draw values from for numeric types.
fn arb_input_map(abi: &Abi, dictionary: &BTreeSet<BigInt>) -> BoxedStrategy<InputMap> {
    let values: Vec<_> = abi
        .parameters
        .iter()
        .map(|param| (Just(param.name.clone()), arb_value_from_abi_type(&param.typ, dictionary)))
        .collect();

    values
        .prop_map(|values| {
            let input_map: InputMap = values.into_iter().collect();
            input_map
        })
        .boxed()
}

/// Create a strategy for generating random values for an [AbiType].
///
/// Uses the `dictionary` for unsigned integer types.
fn arb_value_from_abi_type(
    abi_type: &AbiType,
    dictionary: &BTreeSet<BigInt>,
) -> SBoxedStrategy<InputValue> {
    match abi_type {
        AbiType::Field => proptest::collection::vec(any::<u8>(), 32)
            .prop_map(|bytes| InputValue::Field(FieldElement::from_be_bytes_reduce(&bytes)))
            .sboxed(),
        AbiType::Integer { width, sign } if sign == &Sign::Unsigned => {
            // We've restricted the type system to only allow u64s as the maximum integer type.
            let width = (*width).min(64);
            UintStrategy::new(width as usize, dictionary)
                .prop_map(|uint| InputValue::Field(uint.into()))
                .sboxed()
        }
        AbiType::Integer { width, .. } => {
            let width = (*width).min(64);
            // Based on `FieldElement::to_i128`:
            // * negative integers are represented by the range [p + i128::MIN, p)
            // * positive integers are represented by the range [0, i128::MAX)
            let shift = 2i128.pow(width);
            IntStrategy::new(width as usize)
                .prop_map(move |mut int| {
                    if int < 0 {
                        int += shift;
                    }
                    InputValue::Field(int.into())
                })
                .sboxed()
        }
        AbiType::Boolean => {
            any::<bool>().prop_map(|val| InputValue::Field(FieldElement::from(val))).sboxed()
        }
        AbiType::String { length } => {
            // Strings only allow ASCII characters as each character must be able to be represented by a single byte.
            let string_regex = format!("[[:ascii:]]{{{length}}}");
            proptest::string::string_regex(&string_regex)
                .expect("parsing of regex should always succeed")
                .prop_map(InputValue::String)
                .sboxed()
        }
        AbiType::Array { length, typ } => {
            let length = *length as usize;
            let elements = proptest::collection::vec(
                arb_value_from_abi_type(typ, dictionary),
                length..=length,
            );

            elements.prop_map(InputValue::Vec).sboxed()
        }
        AbiType::Struct { fields, .. } => {
            let fields: Vec<SBoxedStrategy<(String, InputValue)>> = fields
                .iter()
                .map(|(name, typ)| {
                    (Just(name.clone()), arb_value_from_abi_type(typ, dictionary)).sboxed()
                })
                .collect();

            fields
                .prop_map(|fields| {
                    let fields: BTreeMap<_, _> = fields.into_iter().collect();
                    InputValue::Struct(fields)
                })
                .sboxed()
        }
        AbiType::Tuple { fields } => {
            let fields: Vec<_> =
                fields.iter().map(|typ| arb_value_from_abi_type(typ, dictionary)).collect();
            fields.prop_map(InputValue::Vec).sboxed()
        }
    }
}
