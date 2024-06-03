use acir::{
    circuit::{brillig::BrilligOutputs, directives::Directive, Circuit, ExpressionWidth, Opcode},
    native_types::{Expression, Witness},
    AcirField,
};
use indexmap::IndexMap;

mod csat;

pub(crate) use csat::CSatTransformer;

use super::{transform_assert_messages, AcirTransformationMap};

/// Applies [`ProofSystemCompiler`][crate::ProofSystemCompiler] specific optimizations to a [`Circuit`].
pub fn transform<F: AcirField>(
    acir: Circuit<F>,
    expression_width: ExpressionWidth,
) -> (Circuit<F>, AcirTransformationMap) {
    // Track original acir opcode positions throughout the transformation passes of the compilation
    // by applying the modifications done to the circuit opcodes and also to the opcode_positions (delete and insert)
    let acir_opcode_positions = acir.opcodes.iter().enumerate().map(|(i, _)| i).collect();

    let (mut acir, acir_opcode_positions) =
        transform_internal(acir, expression_width, acir_opcode_positions);

    let transformation_map = AcirTransformationMap::new(acir_opcode_positions);

    acir.assert_messages = transform_assert_messages(acir.assert_messages, &transformation_map);

    (acir, transformation_map)
}

/// Applies [`ProofSystemCompiler`][crate::ProofSystemCompiler] specific optimizations to a [`Circuit`].
///
/// Accepts an injected `acir_opcode_positions` to allow transformations to be applied directly after optimizations.
#[tracing::instrument(level = "trace", name = "transform_acir", skip(acir, acir_opcode_positions))]
pub(super) fn transform_internal<F: AcirField>(
    acir: Circuit<F>,
    expression_width: ExpressionWidth,
    acir_opcode_positions: Vec<usize>,
) -> (Circuit<F>, Vec<usize>) {
    let mut transformer = match &expression_width {
        ExpressionWidth::Unbounded => {
            return (acir, acir_opcode_positions);
        }
        ExpressionWidth::Bounded { width } => {
            let mut csat = CSatTransformer::new(*width);
            for value in acir.circuit_arguments() {
                csat.mark_solvable(value);
            }
            csat
        }
    };

    // TODO: the code below is only for CSAT transformer
    // TODO it may be possible to refactor it in a way that we do not need to return early from the r1cs
    // TODO or at the very least, we could put all of it inside of CSatOptimizer pass

    let mut new_acir_opcode_positions: Vec<usize> = Vec::with_capacity(acir_opcode_positions.len());
    // Optimize the assert-zero gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut transformed_opcodes = Vec::new();

    let mut next_witness_index = acir.current_witness_index + 1;
    // maps a normalized expression to the intermediate variable which represents the expression, along with its 'norm'
    // the 'norm' is simply the value of the first non zero coefficient in the expression, taken from the linear terms, or quadratic terms if there is none.
    let mut intermediate_variables: IndexMap<Expression<F>, (F, Witness)> = IndexMap::new();
    for (index, opcode) in acir.opcodes.into_iter().enumerate() {
        match opcode {
            Opcode::AssertZero(arith_expr) => {
                let len = intermediate_variables.len();

                let arith_expr = transformer.transform(
                    arith_expr,
                    &mut intermediate_variables,
                    &mut next_witness_index,
                );

                // Update next_witness counter
                next_witness_index += (intermediate_variables.len() - len) as u32;
                let mut new_opcodes = Vec::new();
                for (g, (norm, w)) in intermediate_variables.iter().skip(len) {
                    // de-normalize
                    let mut intermediate_opcode = g * *norm;
                    // constrain the intermediate opcode to the intermediate variable
                    intermediate_opcode.linear_combinations.push((-F::one(), *w));
                    intermediate_opcode.sort();
                    new_opcodes.push(intermediate_opcode);
                }
                new_opcodes.push(arith_expr);
                for opcode in new_opcodes {
                    new_acir_opcode_positions.push(acir_opcode_positions[index]);
                    transformed_opcodes.push(Opcode::AssertZero(opcode));
                }
            }
            Opcode::BlackBoxFuncCall(ref func) => {
                for witness in func.get_outputs_vec() {
                    transformer.mark_solvable(witness);
                }

                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::Directive(ref directive) => {
                match directive {
                    Directive::ToLeRadix { b, .. } => {
                        for witness in b {
                            transformer.mark_solvable(*witness);
                        }
                    }
                }
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::MemoryInit { .. } => {
                // `MemoryInit` does not write values to the `WitnessMap`
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::MemoryOp { ref op, .. } => {
                for (_, witness1, witness2) in &op.value.mul_terms {
                    transformer.mark_solvable(*witness1);
                    transformer.mark_solvable(*witness2);
                }
                for (_, witness) in &op.value.linear_combinations {
                    transformer.mark_solvable(*witness);
                }
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::BrilligCall { ref outputs, .. } => {
                for output in outputs {
                    match output {
                        BrilligOutputs::Simple(w) => transformer.mark_solvable(*w),
                        BrilligOutputs::Array(v) => {
                            for witness in v {
                                transformer.mark_solvable(*witness);
                            }
                        }
                    }
                }

                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::Call { ref outputs, .. } => {
                for witness in outputs {
                    transformer.mark_solvable(*witness);
                }

                // `Call` does not write values to the `WitnessMap`
                // A separate ACIR function should have its own respective `WitnessMap`
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
        }
    }

    let current_witness_index = next_witness_index - 1;

    let acir = Circuit {
        current_witness_index,
        expression_width,
        opcodes: transformed_opcodes,
        // The transformer does not add new public inputs
        ..acir
    };

    (acir, new_acir_opcode_positions)
}
