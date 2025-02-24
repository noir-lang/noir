use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{
    compiler::optimizers::GeneralOptimizer,
    pwg::{
        arithmetic::ExpressionSolver, blackbox::solve_range_opcode, BrilligSolver,
        BrilligSolverStatus,
    },
};
use acir::{
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::BlackBoxFuncCall,
        Circuit, Opcode,
    },
    native_types::{Expression, Witness, WitnessMap},
};
use acvm_blackbox_solver::StubbedBlackBoxSolver;

/// `ConstantBackpropagationOptimizer` will attempt to determine any constant witnesses within the program.
/// It does this by attempting to solve the program without any inputs (i.e. using an empty witness map),
/// any values which it can determine are then enforced to be constant values.
///
/// The optimizer will then replace any witnesses wherever they appear within the circuit with these constant values.
/// This is repeated until the circuit stabilizes.
pub(crate) struct ConstantBackpropagationOptimizer {
    circuit: Circuit,
}

impl ConstantBackpropagationOptimizer {
    /// Creates a new `ConstantBackpropagationOptimizer`
    pub(crate) fn new(circuit: Circuit) -> Self {
        Self { circuit }
    }

    fn gather_known_witnesses(&self) -> (WitnessMap, BTreeSet<Witness>) {
        // We do not want to affect the circuit's interface so avoid optimizing away these witnesses.
        let mut required_witnesses: BTreeSet<Witness> = self
            .circuit
            .private_parameters
            .union(&self.circuit.public_parameters.0)
            .chain(&self.circuit.return_values.0)
            .copied()
            .collect();

        for opcode in &self.circuit.opcodes {
            match &opcode {
                Opcode::BlackBoxFuncCall(func_call) => {
                    required_witnesses.extend(
                        func_call.get_inputs_vec().into_iter().map(|func_input| func_input.witness),
                    );
                    required_witnesses.extend(func_call.get_outputs_vec());
                }

                Opcode::MemoryInit { init, .. } => {
                    required_witnesses.extend(init);
                }

                Opcode::MemoryOp { op, .. } => {
                    required_witnesses.insert(op.index.to_witness().unwrap());
                    required_witnesses.insert(op.value.to_witness().unwrap());
                }

                _ => (),
            };
        }

        let mut known_witnesses = WitnessMap::new();
        for opcode in self.circuit.opcodes.iter().rev() {
            if let Opcode::AssertZero(expr) = opcode {
                let solve_result = ExpressionSolver::solve(&mut known_witnesses, expr);
                // It doesn't matter what the result is. We expect most opcodes to not be solved successfully so we discard errors.
                // At the same time, if the expression can be solved then we track this by the updates to `known_witnesses`
                drop(solve_result);
            }
        }

        // We want to retain any references to required witnesses so we "forget" these assignments.
        let known_witnesses: BTreeMap<_, _> = known_witnesses
            .into_iter()
            .filter(|(witness, _)| !required_witnesses.contains(witness))
            .collect();

        (known_witnesses.into(), required_witnesses)
    }

    /// Returns a `Circuit` where with any constant witnesses replaced with the constant they resolve to.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn backpropagate_constants(
        circuit: Circuit,
        order_list: Vec<usize>,
    ) -> (Circuit, Vec<usize>) {
        let old_circuit_size = circuit.opcodes.len();

        let optimizer = Self::new(circuit);
        let (circuit, order_list) = optimizer.backpropagate_constants_iteration(order_list);

        let new_circuit_size = circuit.opcodes.len();
        if new_circuit_size < old_circuit_size {
            Self::backpropagate_constants(circuit, order_list)
        } else {
            (circuit, order_list)
        }
    }

    /// Applies a single round of constant backpropagation to a `Circuit`.
    pub(crate) fn backpropagate_constants_iteration(
        mut self,
        order_list: Vec<usize>,
    ) -> (Circuit, Vec<usize>) {
        let (mut known_witnesses, required_witnesses) = self.gather_known_witnesses();

        let opcodes = std::mem::take(&mut self.circuit.opcodes);

        fn remap_expression(known_witnesses: &WitnessMap, expression: Expression) -> Expression {
            GeneralOptimizer::optimize(ExpressionSolver::evaluate(&expression, known_witnesses))
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

                    // Attempt to solve the opcode to see if we can determine the value of any witnesses in the expression.
                    // We only do this _after_ we apply any simplifications to create the new opcode as we want to
                    // keep the constraint on the witness which we are solving for here.
                    let solve_result = ExpressionSolver::solve(&mut known_witnesses, &new_expr);
                    // It doesn't matter what the result is. We expect most opcodes to not be solved successfully so we discard errors.
                    // At the same time, if the expression can be solved then we track this by the updates to `known_witnesses`
                    drop(solve_result);

                    Opcode::AssertZero(new_expr)
                }
                Opcode::Brillig(brillig) => {
                    let remapped_inputs = brillig
                        .inputs
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
                            input @ BrilligInputs::MemoryArray(_) => input,
                        })
                        .collect();

                    let remapped_predicate = brillig
                        .predicate
                        .map(|predicate| remap_expression(&known_witnesses, predicate));

                    let new_brillig = Brillig {
                        inputs: remapped_inputs,
                        predicate: remapped_predicate,
                        ..brillig
                    };

                    let brillig_output_is_required_witness =
                        new_brillig.outputs.iter().any(|output| match output {
                            BrilligOutputs::Simple(witness) => required_witnesses.contains(witness),
                            BrilligOutputs::Array(witness_array) => witness_array
                                .iter()
                                .any(|witness| required_witnesses.contains(witness)),
                        });

                    if brillig_output_is_required_witness {
                        // If one of the brillig opcode's outputs is a required witness then we can't remove the opcode. In this case we can't replace
                        // all of the uses of this witness with the calculated constant so we'll be attempting to use an uninitialized witness.
                        //
                        // We then do not attempt execution of this opcode and just simplify the inputs.
                        Opcode::Brillig(new_brillig)
                    } else if let Ok(mut solver) = BrilligSolver::new(
                        &known_witnesses,
                        &HashMap::new(),
                        &new_brillig,
                        &StubbedBlackBoxSolver,
                        idx,
                    ) {
                        match solver.solve() {
                            Ok(BrilligSolverStatus::Finished) => {
                                // Write execution outputs
                                match solver.finalize(&mut known_witnesses, &new_brillig) {
                                    Ok(()) => {
                                        // If we've managed to execute the brillig opcode at compile time, we can now just write in the
                                        // results as constants for the rest of the circuit.
                                        continue;
                                    }
                                    _ => Opcode::Brillig(new_brillig),
                                }
                            }
                            Ok(BrilligSolverStatus::InProgress) => unreachable!(
                                "Solver should either finish, block on foreign call, or error."
                            ),
                            Ok(BrilligSolverStatus::ForeignCallWait(_)) | Err(_) => {
                                Opcode::Brillig(new_brillig)
                            }
                        }
                    } else {
                        Opcode::Brillig(new_brillig)
                    }
                }

                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE { input }) => {
                    if solve_range_opcode(&known_witnesses, &input).is_ok() {
                        continue;
                    } else {
                        opcode
                    }
                }

                Opcode::BlackBoxFuncCall(_)
                | Opcode::MemoryOp { .. }
                | Opcode::MemoryInit { .. } => opcode,
            };

            new_opcodes.push(new_opcode);
            new_order_list.push(order_list[idx]);
        }

        self.circuit.opcodes = new_opcodes;

        (self.circuit, new_order_list)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::compiler::optimizers::constant_backpropagation::ConstantBackpropagationOptimizer;
    use acir::{
        brillig::MemoryAddress,
        circuit::{
            brillig::{Brillig, BrilligOutputs},
            opcodes::{BlackBoxFuncCall, FunctionInput},
            Circuit, ExpressionWidth, Opcode, PublicInputs,
        },
        native_types::Witness,
    };
    use brillig_vm::brillig::Opcode as BrilligOpcode;

    fn test_circuit(opcodes: Vec<Opcode>) -> Circuit {
        Circuit {
            current_witness_index: 1,
            expression_width: ExpressionWidth::Bounded { width: 4 },
            opcodes,
            private_parameters: BTreeSet::new(),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Default::default(),
        }
    }

    #[test]
    fn retain_brillig_with_required_witness_outputs() {
        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: Vec::new(),
            outputs: vec![BrilligOutputs::Simple(Witness(1))],
            bytecode: vec![
                BrilligOpcode::Const {
                    destination: MemoryAddress(0),
                    bit_size: 32,
                    value: 1u128.into(),
                },
                BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 1 },
            ],
            predicate: None,
        });
        let blackbox_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput { witness: Witness(1), num_bits: 64 },
            rhs: FunctionInput { witness: Witness(2), num_bits: 64 },
            output: Witness(3),
        });

        let opcodes = vec![brillig_opcode, blackbox_opcode];
        // The optimizer should keep the lowest bit size range constraint
        let circuit = test_circuit(opcodes);
        let acir_opcode_positions = circuit.opcodes.iter().enumerate().map(|(i, _)| i).collect();
        let optimizer = ConstantBackpropagationOptimizer::new(circuit);

        let (optimized_circuit, _) =
            optimizer.backpropagate_constants_iteration(acir_opcode_positions);

        assert_eq!(
            optimized_circuit.opcodes.len(),
            2,
            "The brillig opcode should not be removed as the output is needed as a witness"
        );
    }
}
