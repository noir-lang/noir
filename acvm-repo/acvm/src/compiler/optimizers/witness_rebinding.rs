use std::collections::{BTreeSet, HashMap};

use crate::compiler::optimizers::GeneralOptimizer;
use acir::{
    circuit::{
        brillig::{BrilligInputs, BrilligOutputs},
        directives::Directive,
        opcodes::{BlackBoxFuncCall, FunctionInput, MemOp},
        Circuit, Opcode,
    },
    native_types::{Expression, Witness},
    AcirField,
};

/// `WitnessRebindingOptimizer` will remove redundant range constraints.
///
/// # Example
///
/// Suppose we had the following circuit where witness 0 is an input and
/// witness 2 is a return value:
///
/// ```
/// EXPR [ (-1, _0) (1, _1) 0 ]
/// EXPR [ (-1, _1) (1, _2) 0 ]
/// ```
///
/// While it's not possible to determine the values of any of the witnesses, it's clear that they will all be equal.
/// As witness 1 is internal to the circuit we can then remove it and replace any references to it with witness 0.
///
/// ```
/// EXPR [ (-1, _0) (1, _2) 0 ]
/// ```
///
/// This optimization pass looks for [`Opcode::AssertZero`] opcodes of this form and replaces references and replaces
/// all references to one of these witnesses with the other to apply this equality in all other opcodes.
pub(crate) struct WitnessRebindingOptimizer<F> {
    circuit: Circuit<F>,
}

impl<F: AcirField> WitnessRebindingOptimizer<F> {
    /// Creates a new `WitnessRebindingOptimizer`
    pub(crate) fn new(circuit: Circuit<F>) -> Self {
        Self { circuit }
    }

    fn gather_equivalent_witnesses(&self) -> HashMap<Witness, Witness> {
        // We do not want to affect the circuit's interface so avoid optimizing away these witnesses.
        let mut required_witnesses: BTreeSet<Witness> = self
            .circuit
            .private_parameters
            .union(&self.circuit.public_parameters.0)
            .chain(&self.circuit.return_values.0)
            .copied()
            .collect();

        let mut equivalent_witnesses: HashMap<Witness, Witness> = HashMap::new();
        for opcode in self.circuit.opcodes.iter().rev() {
            #[allow(clippy::single_match)]
            match &opcode {
                Opcode::AssertZero(Expression { mul_terms, linear_combinations, q_c })
                    if mul_terms.is_empty() && q_c.is_zero() && linear_combinations.len() == 2 =>
                {
                    let [(k1, w1), (k2, w2)]: [(F, Witness); 2] =
                        linear_combinations.clone().try_into().unwrap();
                    if k1 * k2 == -F::one() {
                        match (required_witnesses.contains(&w1), required_witnesses.contains(&w2)) {
                            (false, false) | (true, false) => {
                                equivalent_witnesses.insert(w2, w1);
                            }
                            (false, true) => {
                                equivalent_witnesses.insert(w1, w2);
                            }
                            (true, true) => (),
                        };
                    }
                }

                Opcode::MemoryOp { op, .. } if op.operation.is_zero() => {
                    required_witnesses.insert(op.value.to_witness().unwrap());
                    equivalent_witnesses.remove(&op.value.to_witness().unwrap());
                }

                _ => (),
            };
        }

        equivalent_witnesses
    }

    /// Returns a `Circuit` where any witnesses assigned to be equal to a another witness has been replaced.
    pub(crate) fn rebind_witnesses(
        circuit: Circuit<F>,
        order_list: Vec<usize>,
    ) -> (Circuit<F>, Vec<usize>) {
        let old_circuit_size = circuit.opcodes.len();

        let optimizer = Self::new(circuit);
        let (circuit, order_list) = optimizer.rebind_witnesses_iteration(order_list);

        let new_circuit_size = circuit.opcodes.len();
        if new_circuit_size < old_circuit_size {
            Self::rebind_witnesses(circuit, order_list)
        } else {
            (circuit, order_list)
        }
    }

    pub(crate) fn rebind_witnesses_iteration(
        mut self,
        order_list: Vec<usize>,
    ) -> (Circuit<F>, Vec<usize>) {
        let known_witnesses = self.gather_equivalent_witnesses();

        let opcodes = std::mem::take(&mut self.circuit.opcodes);

        fn resolve_witness(
            witness_mapping: &HashMap<Witness, Witness>,
            witness: Witness,
        ) -> Witness {
            if let Some(mapped_witness) = witness_mapping.get(&witness) {
                resolve_witness(witness_mapping, *mapped_witness)
            } else {
                witness
            }
        }

        fn remap_expression<F: AcirField>(
            witness_mapping: &HashMap<Witness, Witness>,
            expression: Expression<F>,
        ) -> Expression<F> {
            GeneralOptimizer::optimize(Expression {
                mul_terms: expression
                    .mul_terms
                    .into_iter()
                    .map(|(f, w1, w2)| {
                        (
                            f,
                            resolve_witness(witness_mapping, w1),
                            resolve_witness(witness_mapping, w2),
                        )
                    })
                    .collect(),
                linear_combinations: expression
                    .linear_combinations
                    .into_iter()
                    .map(|(f, w)| (f, resolve_witness(witness_mapping, w)))
                    .collect(),
                q_c: expression.q_c,
            })
        }

        fn remap_function_input(
            witness_mapping: &HashMap<Witness, Witness>,
            function_input: FunctionInput,
        ) -> FunctionInput {
            FunctionInput {
                witness: resolve_witness(witness_mapping, function_input.witness),
                num_bits: function_input.num_bits,
            }
        }

        let mut new_order_list = Vec::with_capacity(order_list.len());
        let mut new_opcodes = Vec::with_capacity(opcodes.len());
        for (idx, opcode) in opcodes.into_iter().enumerate() {
            let new_opcode = match opcode {
                Opcode::AssertZero(expression) => {
                    let new_expr = remap_expression(&known_witnesses, expression);
                    if new_expr.is_zero() {
                        continue;
                    }
                    Opcode::AssertZero(new_expr)
                }
                Opcode::BlackBoxFuncCall(func) => Opcode::BlackBoxFuncCall(match func {
                    BlackBoxFuncCall::AND { lhs, rhs, output } => BlackBoxFuncCall::AND {
                        lhs: remap_function_input(&known_witnesses, lhs),
                        rhs: remap_function_input(&known_witnesses, rhs),
                        output: resolve_witness(&known_witnesses, output),
                    },
                    BlackBoxFuncCall::XOR { lhs, rhs, output } => BlackBoxFuncCall::XOR {
                        lhs: remap_function_input(&known_witnesses, lhs),
                        rhs: remap_function_input(&known_witnesses, rhs),
                        output: resolve_witness(&known_witnesses, output),
                    },
                    BlackBoxFuncCall::RANGE { input } => BlackBoxFuncCall::RANGE {
                        input: remap_function_input(&known_witnesses, input),
                    },
                    BlackBoxFuncCall::SHA256 { inputs, outputs } => BlackBoxFuncCall::SHA256 {
                        inputs: inputs
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect(),
                        outputs: outputs
                            .into_iter()
                            .map(|output| resolve_witness(&known_witnesses, output))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                    },
                    BlackBoxFuncCall::Blake2s { inputs, outputs } => BlackBoxFuncCall::Blake2s {
                        inputs: inputs
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect(),
                        outputs: outputs
                            .into_iter()
                            .map(|output| resolve_witness(&known_witnesses, output))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                    },
                    BlackBoxFuncCall::Blake3 { inputs, outputs } => BlackBoxFuncCall::Blake3 {
                        inputs: inputs
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect(),
                        outputs: outputs
                            .into_iter()
                            .map(|output| resolve_witness(&known_witnesses, output))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                    },
                    BlackBoxFuncCall::SchnorrVerify {
                        public_key_x,
                        public_key_y,
                        signature,
                        message,
                        output,
                    } => BlackBoxFuncCall::SchnorrVerify {
                        public_key_x: remap_function_input(&known_witnesses, public_key_x),
                        public_key_y: remap_function_input(&known_witnesses, public_key_y),
                        signature: signature
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        message: message
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect(),
                        output: resolve_witness(&known_witnesses, output),
                    },
                    BlackBoxFuncCall::PedersenCommitment { inputs, domain_separator, outputs } => {
                        BlackBoxFuncCall::PedersenCommitment {
                            inputs: inputs
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect(),
                            domain_separator,
                            outputs: (
                                resolve_witness(&known_witnesses, outputs.0),
                                resolve_witness(&known_witnesses, outputs.1),
                            ),
                        }
                    }
                    BlackBoxFuncCall::PedersenHash { inputs, domain_separator, output } => {
                        BlackBoxFuncCall::PedersenHash {
                            inputs: inputs
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect(),
                            domain_separator,
                            output: resolve_witness(&known_witnesses, output),
                        }
                    }
                    BlackBoxFuncCall::EcdsaSecp256k1 {
                        public_key_x,
                        public_key_y,
                        signature,
                        hashed_message,
                        output,
                    } => BlackBoxFuncCall::EcdsaSecp256k1 {
                        public_key_x: public_key_x
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        public_key_y: public_key_y
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        signature: signature
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        hashed_message: hashed_message
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        output: resolve_witness(&known_witnesses, output),
                    },
                    BlackBoxFuncCall::EcdsaSecp256r1 {
                        public_key_x,
                        public_key_y,
                        signature,
                        hashed_message,
                        output,
                    } => BlackBoxFuncCall::EcdsaSecp256r1 {
                        public_key_x: public_key_x
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        public_key_y: public_key_y
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        signature: signature
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        hashed_message: hashed_message
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap(),
                        output: resolve_witness(&known_witnesses, output),
                    },
                    BlackBoxFuncCall::Keccak256 { inputs, var_message_size, outputs } => {
                        BlackBoxFuncCall::Keccak256 {
                            inputs: inputs
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect(),
                            var_message_size: remap_function_input(
                                &known_witnesses,
                                var_message_size,
                            ),
                            outputs: outputs
                                .into_iter()
                                .map(|output| resolve_witness(&known_witnesses, output))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                        }
                    }
                    BlackBoxFuncCall::Keccakf1600 { inputs, outputs } => {
                        BlackBoxFuncCall::Keccakf1600 {
                            inputs: inputs
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                            outputs: outputs
                                .into_iter()
                                .map(|output| resolve_witness(&known_witnesses, output))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                        }
                    }
                    BlackBoxFuncCall::RecursiveAggregation {
                        verification_key,
                        proof,
                        public_inputs,
                        key_hash,
                    } => BlackBoxFuncCall::RecursiveAggregation {
                        verification_key: verification_key
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect(),
                        proof: proof
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect(),
                        public_inputs: public_inputs
                            .into_iter()
                            .map(|input| remap_function_input(&known_witnesses, input))
                            .collect(),
                        key_hash: remap_function_input(&known_witnesses, key_hash),
                    },
                    BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, outputs } => {
                        BlackBoxFuncCall::EmbeddedCurveAdd {
                            input1: Box::new([
                                remap_function_input(&known_witnesses, input1[0]),
                                remap_function_input(&known_witnesses, input1[1]),
                                remap_function_input(&known_witnesses, input1[2]),
                            ]),
                            input2: Box::new([
                                remap_function_input(&known_witnesses, input2[0]),
                                remap_function_input(&known_witnesses, input2[1]),
                                remap_function_input(&known_witnesses, input2[2]),
                            ]),
                            outputs: (
                                resolve_witness(&known_witnesses, outputs.0),
                                resolve_witness(&known_witnesses, outputs.1),
                                resolve_witness(&known_witnesses, outputs.2),
                            ),
                        }
                    }
                    BlackBoxFuncCall::BigIntAdd { lhs, rhs, output } => {
                        BlackBoxFuncCall::BigIntAdd { lhs, rhs, output }
                    }
                    BlackBoxFuncCall::BigIntSub { lhs, rhs, output } => {
                        BlackBoxFuncCall::BigIntSub { lhs, rhs, output }
                    }
                    BlackBoxFuncCall::BigIntMul { lhs, rhs, output } => {
                        BlackBoxFuncCall::BigIntMul { lhs, rhs, output }
                    }
                    BlackBoxFuncCall::BigIntDiv { lhs, rhs, output } => {
                        BlackBoxFuncCall::BigIntDiv { lhs, rhs, output }
                    }
                    BlackBoxFuncCall::BigIntFromLeBytes { inputs, modulus, output } => {
                        BlackBoxFuncCall::BigIntFromLeBytes {
                            inputs: inputs
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect(),
                            modulus,
                            output,
                        }
                    }
                    BlackBoxFuncCall::BigIntToLeBytes { input, outputs } => {
                        BlackBoxFuncCall::BigIntToLeBytes {
                            input,
                            outputs: outputs
                                .into_iter()
                                .map(|expr| resolve_witness(&known_witnesses, expr))
                                .collect(),
                        }
                    }
                    BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs, len } => {
                        BlackBoxFuncCall::Poseidon2Permutation {
                            inputs: inputs
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect(),
                            outputs: outputs
                                .into_iter()
                                .map(|expr| resolve_witness(&known_witnesses, expr))
                                .collect(),
                            len,
                        }
                    }
                    BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs } => {
                        BlackBoxFuncCall::Sha256Compression {
                            inputs: inputs
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                            hash_values: hash_values
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                            outputs: outputs
                                .into_iter()
                                .map(|expr| resolve_witness(&known_witnesses, expr))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                        }
                    }
                    BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs } => {
                        BlackBoxFuncCall::AES128Encrypt {
                            inputs: inputs
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect(),
                            iv: iv
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                            key: key
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect::<Vec<_>>()
                                .try_into()
                                .unwrap(),
                            outputs: outputs
                                .into_iter()
                                .map(|output| resolve_witness(&known_witnesses, output))
                                .collect(),
                        }
                    }
                    BlackBoxFuncCall::MultiScalarMul { points, scalars, outputs } => {
                        BlackBoxFuncCall::MultiScalarMul {
                            points: points
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect(),
                            scalars: scalars
                                .into_iter()
                                .map(|input| remap_function_input(&known_witnesses, input))
                                .collect(),
                            outputs: (
                                resolve_witness(&known_witnesses, outputs.0),
                                resolve_witness(&known_witnesses, outputs.1),
                                resolve_witness(&known_witnesses, outputs.2),
                            ),
                        }
                    }
                }),
                Opcode::Directive(directive) => acir::circuit::Opcode::Directive(match directive {
                    Directive::ToLeRadix { a, b, radix } => Directive::ToLeRadix {
                        a: remap_expression(&known_witnesses, a),
                        b: b.into_iter()
                            .map(|expr| resolve_witness(&known_witnesses, expr))
                            .collect(),
                        radix,
                    },
                }),
                Opcode::MemoryOp { block_id, op, predicate } => Opcode::MemoryOp {
                    block_id,
                    op: MemOp {
                        operation: remap_expression(&known_witnesses, op.operation),
                        index: remap_expression(&known_witnesses, op.index),
                        value: remap_expression(&known_witnesses, op.value),
                    },
                    predicate: predicate
                        .map(|predicate| remap_expression(&known_witnesses, predicate)),
                },
                Opcode::MemoryInit { block_id, init, block_type } => Opcode::MemoryInit {
                    block_id,
                    init: init
                        .into_iter()
                        .map(|wit| resolve_witness(&known_witnesses, wit))
                        .collect(),
                    block_type,
                },
                Opcode::Call { id, inputs, outputs, predicate } => Opcode::Call {
                    id,
                    inputs: inputs
                        .into_iter()
                        .map(|wit| resolve_witness(&known_witnesses, wit))
                        .collect(),
                    outputs: outputs
                        .into_iter()
                        .map(|wit| resolve_witness(&known_witnesses, wit))
                        .collect(),
                    predicate: predicate.map(|expr| remap_expression(&known_witnesses, expr)),
                },
                Opcode::BrilligCall { id, inputs, outputs, predicate } => Opcode::BrilligCall {
                    id,
                    inputs: inputs
                        .into_iter()
                        .map(|input| match input {
                            BrilligInputs::Single(expr) => {
                                BrilligInputs::Single(remap_expression(&known_witnesses, expr))
                            }
                            BrilligInputs::Array(expr_array) => {
                                let new_input: Vec<_> = expr_array
                                    .into_iter()
                                    .map(|expr| remap_expression(&known_witnesses, expr))
                                    .collect();

                                BrilligInputs::Array(new_input)
                            }
                            mem_array @ BrilligInputs::MemoryArray(_) => mem_array,
                        })
                        .collect(),
                    outputs: outputs
                        .into_iter()
                        .map(|output| match output {
                            BrilligOutputs::Simple(witness) => {
                                BrilligOutputs::Simple(resolve_witness(&known_witnesses, witness))
                            }
                            BrilligOutputs::Array(witness_array) => {
                                let new_output: Vec<_> = witness_array
                                    .into_iter()
                                    .map(|expr| resolve_witness(&known_witnesses, expr))
                                    .collect();

                                BrilligOutputs::Array(new_output)
                            }
                        })
                        .collect(),
                    predicate: predicate.map(|expr| remap_expression(&known_witnesses, expr)),
                },
            };

            new_opcodes.push(new_opcode);
            new_order_list.push(order_list[idx]);
        }

        self.circuit.opcodes = new_opcodes;

        (self.circuit, new_order_list)
    }
}
