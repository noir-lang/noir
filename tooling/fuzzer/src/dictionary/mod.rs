//! This module defines how to build a dictionary of values which are likely to be correspond
//! to significant inputs during fuzzing by inspecting the [Program] being fuzzed.
//!
//! This dictionary can be fed into the fuzzer's [strategy][proptest::strategy::Strategy] in order to bias it towards
//! generating these values to ensure they get proper coverage.
use std::collections::HashSet;

use acvm::{
    acir::{
        circuit::{
            brillig::{BrilligBytecode, BrilligInputs},
            opcodes::{BlackBoxFuncCall, ConstantOrWitnessEnum},
            Circuit, Opcode, Program,
        },
        native_types::Expression,
    },
    brillig_vm::brillig::Opcode as BrilligOpcode,
    AcirField,
};

/// Constructs a [HashSet<F>] of values pulled from a [Program<F>] which are likely to be correspond
/// to significant inputs during fuzzing.
pub(super) fn build_dictionary_from_program<F: AcirField>(program: &Program<F>) -> HashSet<F> {
    let constrained_dictionaries = program.functions.iter().map(build_dictionary_from_circuit);
    let unconstrained_dictionaries =
        program.unconstrained_functions.iter().map(build_dictionary_from_unconstrained_function);
    let dictionaries = constrained_dictionaries.chain(unconstrained_dictionaries);

    let mut constants: HashSet<F> = HashSet::new();
    for dictionary in dictionaries {
        constants.extend(dictionary);
    }
    constants
}

fn build_dictionary_from_circuit<F: AcirField>(circuit: &Circuit<F>) -> HashSet<F> {
    let mut constants: HashSet<F> = HashSet::new();

    fn insert_expr<F: AcirField>(dictionary: &mut HashSet<F>, expr: &Expression<F>) {
        let quad_coefficients = expr.mul_terms.iter().map(|(k, _, _)| *k);
        let linear_coefficients = expr.linear_combinations.iter().map(|(k, _)| *k);
        let coefficients = linear_coefficients.chain(quad_coefficients);

        dictionary.extend(coefficients.clone());
        dictionary.insert(expr.q_c);

        // We divide the constant term by any coefficients in the expression to aid solving constraints such as `2 * x - 4 == 0`.
        let scaled_constants = coefficients.map(|coefficient| expr.q_c / coefficient);
        dictionary.extend(scaled_constants);
    }

    fn insert_array_len<F: AcirField, T>(dictionary: &mut HashSet<F>, array: &[T]) {
        let array_length = array.len() as u128;
        dictionary.insert(F::from(array_length));
        dictionary.insert(F::from(array_length - 1));
    }

    for opcode in &circuit.opcodes {
        match opcode {
            Opcode::AssertZero(expr)
            | Opcode::Call { predicate: Some(expr), .. }
            | Opcode::MemoryOp { predicate: Some(expr), .. } => insert_expr(&mut constants, expr),

            Opcode::MemoryInit { init, .. } => insert_array_len(&mut constants, init),

            Opcode::BrilligCall { inputs, predicate, .. } => {
                for input in inputs {
                    match input {
                        BrilligInputs::Single(expr) => insert_expr(&mut constants, expr),
                        BrilligInputs::Array(exprs) => {
                            exprs.iter().for_each(|expr| insert_expr(&mut constants, expr));
                            insert_array_len(&mut constants, exprs);
                        }
                        BrilligInputs::MemoryArray(_) => (),
                    }
                }
                if let Some(predicate) = predicate {
                    insert_expr(&mut constants, predicate)
                }
            }

            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { input })
                if matches!(input.input(), ConstantOrWitnessEnum::Constant(..)) =>
            {
                match input.input() {
                    ConstantOrWitnessEnum::Constant(c) => {
                        let field = 1u128.wrapping_shl(input.num_bits());
                        constants.insert(F::from(field));
                        constants.insert(F::from(field - 1));
                        constants.insert(c);
                    }
                    _ => {
                        let field = 1u128.wrapping_shl(input.num_bits());
                        constants.insert(F::from(field));
                        constants.insert(F::from(field - 1));
                    }
                }
            }

            _ => (),
        }
    }

    constants
}

fn build_dictionary_from_unconstrained_function<F: AcirField>(
    function: &BrilligBytecode<F>,
) -> HashSet<F> {
    let mut constants: HashSet<F> = HashSet::new();

    for opcode in &function.bytecode {
        match opcode {
            BrilligOpcode::Cast { bit_size, .. } => {
                let bit_size = bit_size.to_u32::<F>();

                let field = 1u128.wrapping_shl(bit_size);
                constants.insert(F::from(field));
                constants.insert(F::from(field - 1));
            }
            BrilligOpcode::Const { bit_size, value, .. } => {
                let bit_size = bit_size.to_u32::<F>();

                constants.insert(*value);

                let field = 1u128.wrapping_shl(bit_size);
                constants.insert(F::from(field));
                constants.insert(F::from(field - 1));
            }
            _ => (),
        }
    }

    constants
}
