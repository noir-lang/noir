use std::collections::{BTreeMap, BTreeSet};

use crate::{
    blackbox_solver::{BlackBoxFunctionSolver, BlackBoxResolutionError},
    pwg::blackbox::contains_all_inputs,
};
use acir::{
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::BlackBoxFuncCall,
        Circuit, Opcode,
    },
    native_types::{Expression, Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};

use crate::pwg::{arithmetic::ArithmeticSolver, brillig::BrilligSolver, OpcodeResolutionError};

struct NullBbSolver;

impl BlackBoxFunctionSolver for NullBbSolver {
    fn schnorr_verify(
        &self,
        _public_key_x: &FieldElement,
        _public_key_y: &FieldElement,
        _signature: &[u8],
        _message: &[u8],
    ) -> Result<bool, BlackBoxResolutionError> {
        Err(BlackBoxResolutionError::Unsupported(BlackBoxFunc::SchnorrVerify))
    }
    fn pedersen(
        &self,
        _inputs: &[FieldElement],
        _domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        Err(BlackBoxResolutionError::Unsupported(BlackBoxFunc::Pedersen))
    }
    fn fixed_base_scalar_mul(
        &self,
        _low: &FieldElement,
        _high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        Err(BlackBoxResolutionError::Unsupported(BlackBoxFunc::FixedBaseScalarMul))
    }
}

/// `RangeOptimizer` will remove redundant range constraints.
///
/// # Example
///
/// Suppose we had the following pseudo-code:
///
/// ```noir
/// let z1 = x as u16;
/// let z2 = x as u32;
/// ```
/// It is clear that if `x` fits inside of a 16-bit integer,
/// it must also fit inside of a 32-bit integer.
///
/// The generated ACIR may produce two range opcodes however;
/// - One for the 16 bit range constraint of `x`
/// - One for the 32-bit range constraint of `x`
///
/// This optimization pass will keep the 16-bit range constraint
/// and remove the 32-bit range constraint opcode.
pub(crate) struct ConstantBackpropOptimizer {
    circuit: Circuit,
}

fn is_simple_witness_assignment(expr: &Expression) -> bool {
    expr.is_degree_one_univariate()
}

impl ConstantBackpropOptimizer {
    /// Creates a new `ConstantBackpropOptimizer`
    pub(crate) fn new(circuit: Circuit) -> Self {
        Self { circuit }
    }

    fn gather_constants(&self) -> WitnessMap {
        // We do not want to affect the circuit's interface so avoid optimizing away these witnesses.
        let required_witnesses: BTreeSet<Witness> = self
            .circuit
            .private_parameters
            .union(&self.circuit.public_parameters.0)
            .chain(&self.circuit.return_values.0)
            .copied()
            .collect();

        let mut known_witnesses = WitnessMap::new();
        for opcode in self.circuit.opcodes.iter().rev() {
            #[allow(clippy::single_match)]
            match &opcode {
                Opcode::Arithmetic(expr) => {
                    match ArithmeticSolver::solve(&mut known_witnesses, expr) {
                        Ok(()) | Err(OpcodeResolutionError::OpcodeNotSolvable(_)) => (),
                        Err(_) => todo!(),
                    }
                }
                _ => (),
            };
        }

        // We want to retain any references to required witnesses so we "forget" these assignments.
        let known_witnesses: BTreeMap<_, _> = known_witnesses
            .into_iter()
            .filter(|(witness, _)| !required_witnesses.contains(witness))
            .collect();

        known_witnesses.into()
    }

    /// Returns a `Circuit` where each Witness is only range constrained
    /// once to the lowest number `bit size` possible.
    pub(crate) fn backpropagate_constants(
        circuit: Circuit,
        order_list: Vec<usize>,
    ) -> (Circuit, Vec<usize>) {
        let num_opcodes = order_list.len();

        let optimizer = Self::new(circuit);
        optimizer.gather_constants();

        let (circuit, order_list) = optimizer.backpropagate_constants_iteration(order_list);

        if order_list.len() == num_opcodes {
            (circuit, order_list)
        } else {
            Self::backpropagate_constants(circuit, order_list)
        }
    }

    /// Returns a `Circuit` where each Witness is only range constrained
    /// once to the lowest number `bit size` possible.
    pub(crate) fn backpropagate_constants_iteration(
        self,
        order_list: Vec<usize>,
    ) -> (Circuit, Vec<usize>) {
        let mut known_witnesses = self.gather_constants();

        let Circuit {
            current_witness_index,
            opcodes,
            private_parameters,
            public_parameters,
            return_values,
            assert_messages,
        } = self.circuit;

        let mut new_opcodes = Vec::with_capacity(opcodes.len());
        let mut new_order_list = Vec::with_capacity(order_list.len());
        for (idx, opcode) in opcodes.into_iter().enumerate() {
            match &opcode {
                // We don't want to optimize away any witness assignments as this can result in memoryops becoming unsolvable.
                Opcode::Arithmetic(expr) if !is_simple_witness_assignment(expr) => {
                    let simplified_expr = ArithmeticSolver::evaluate(expr, &known_witnesses);

                    if simplified_expr.is_const() {
                        // opcode is a no-op and so can be skipped.
                    } else {
                        // TODO: We currently retain all references to required witnesses.
                        // We could replace any instances in addition to that which constrains the required witness with constants.
                        new_opcodes.push(Opcode::Arithmetic(simplified_expr));
                        new_order_list.push(order_list[idx]);
                    }
                }
                Opcode::Brillig(brillig) => {
                    // If all outputs of the Brillig opcode are known then we can remove it.
                    if brillig.outputs.iter().all(|output| match output {
                        BrilligOutputs::Simple(witness) => known_witnesses.contains_key(witness),
                        BrilligOutputs::Array(witnesses) => {
                            witnesses.iter().all(|witness| known_witnesses.contains_key(witness))
                        }
                    }) {
                        continue;
                    }

                    // Otherwise we can apply any simplifications which we know to the inputs and predicate.
                    // Note this could lead to a situation where the brillig opcode has constant inputs.

                    let simplified_inputs = brillig
                        .inputs
                        .iter()
                        .map(|input| match input {
                            BrilligInputs::Single(expr) => BrilligInputs::Single(
                                ArithmeticSolver::evaluate(expr, &known_witnesses),
                            ),
                            BrilligInputs::Array(expr_array) => {
                                let new_input: Vec<_> = expr_array
                                    .iter()
                                    .map(|expr| ArithmeticSolver::evaluate(expr, &known_witnesses))
                                    .collect();

                                BrilligInputs::Array(new_input)
                            }
                        })
                        .collect();

                    let simplified_predicate = brillig
                        .predicate
                        .as_ref()
                        .map(|predicate| ArithmeticSolver::evaluate(predicate, &known_witnesses));

                    let new_brillig = Brillig {
                        inputs: simplified_inputs,
                        outputs: brillig.outputs.clone(),
                        foreign_call_results: brillig.foreign_call_results.clone(),
                        bytecode: brillig.bytecode.clone(),
                        predicate: simplified_predicate,
                    };

                    // If the opcode's inputs are known constants then we can attempt to execute the Brillig opcode.
                    // If successful we can insert the results as constants in the circuit.
                    match BrilligSolver::solve(
                        &mut known_witnesses,
                        &new_brillig,
                        &NullBbSolver,
                        idx,
                    ) {
                        Ok(None) => {
                            // Brillig opcode has been fully resolved
                            continue;
                        }
                        Ok(Some(_)) | Err(_) => {
                            new_opcodes.push(Opcode::Brillig(new_brillig));
                            new_order_list.push(order_list[idx]);
                        }
                    };
                }

                // Drop any range opcodes for which we know the input
                Opcode::BlackBoxFuncCall(bb_function)
                    if contains_all_inputs(&known_witnesses, &bb_function.get_inputs_vec())
                        && matches!(bb_function, BlackBoxFuncCall::RANGE { .. }) =>
                {
                    continue
                }
                opcode => {
                    new_opcodes.push(opcode.clone());
                    new_order_list.push(order_list[idx]);
                }
            };
        }

        (
            Circuit {
                current_witness_index,
                opcodes: new_opcodes,
                private_parameters,
                public_parameters,
                return_values,
                assert_messages,
            },
            new_order_list,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::ConstantBackpropOptimizer;
    use acir::{
        brillig,
        circuit::{
            brillig::{Brillig, BrilligInputs, BrilligOutputs},
            Circuit, Opcode, PublicInputs,
        },
        native_types::{Expression, Witness},
        FieldElement,
    };

    // This test case
    #[test]
    fn backpropagate_constants() {
        let circuit = Circuit {
            current_witness_index: 4,
            opcodes: vec![
                Opcode::Arithmetic(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(1)),
                        (-FieldElement::one(), Witness(2)),
                    ],
                    q_c: -FieldElement::one(),
                }),
                Opcode::Brillig(Brillig {
                    inputs: vec![BrilligInputs::Single(Witness(2).into())],
                    outputs: vec![BrilligOutputs::Simple(Witness(3))],
                    foreign_call_results: Vec::new(),
                    bytecode: vec![brillig::Opcode::Stop], // Brillig bytecode is irrelevant to this optimization.
                    predicate: None,
                }),
                Opcode::Arithmetic(Expression {
                    mul_terms: vec![(FieldElement::one(), Witness(2), Witness(3))],
                    linear_combinations: vec![(FieldElement::one(), Witness(4))],
                    q_c: -FieldElement::one(),
                }),
                Opcode::Arithmetic(Expression {
                    mul_terms: vec![(FieldElement::one(), Witness(2), Witness(4))],
                    linear_combinations: Vec::new(),
                    q_c: FieldElement::zero(),
                }),
                Opcode::Arithmetic(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![(FieldElement::one(), Witness(4))],
                    q_c: -FieldElement::one(),
                }),
            ],
            private_parameters: BTreeSet::from([Witness(1)]),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Vec::new(),
        };

        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();

        let optimizer = ConstantBackpropOptimizer::new(circuit);
        let (optimized_circuit, new_opcode_labels) =
            optimizer.backpropagate_constants(acir_opcode_positions);

        println!("{}", optimized_circuit);
        println!("{:?}", new_opcode_labels);
    }

    #[test]
    fn fork() {
        let circuit = Circuit {
            current_witness_index: 4,
            opcodes: vec![
                Opcode::Arithmetic(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(1)),
                        (-FieldElement::one(), Witness(3)),
                    ],
                    q_c: FieldElement::zero(),
                }),
                Opcode::Arithmetic(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(2)),
                        (-FieldElement::one(), Witness(4)),
                    ],
                    q_c: FieldElement::zero(),
                }),
                Opcode::Arithmetic(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(3)),
                        (-FieldElement::one(), Witness(4)),
                    ],
                    q_c: FieldElement::zero(),
                }),
            ],
            private_parameters: BTreeSet::from([Witness(1)]),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Vec::new(),
        };

        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();

        let optimizer = ConstantBackpropOptimizer::new(circuit);
        let (optimized_circuit, new_opcode_labels) =
            optimizer.backpropagate_constants(acir_opcode_positions);

        println!("{}", optimized_circuit);
        println!("{:?}", new_opcode_labels);
    }
}
