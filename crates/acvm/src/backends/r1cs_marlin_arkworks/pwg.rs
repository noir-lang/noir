use crate::{
    pwg::{arithmetic::ArithmeticSolver, logic::LogicSolver},
    PartialWitnessGenerator,
};
use acir::{circuit::Gate, native_types::Witness};
use noir_field::FieldElement;
use std::collections::BTreeMap;

mod gadget_call;

use self::gadget_call::GadgetCaller;
use super::Marlin;

impl PartialWitnessGenerator for Marlin {
    fn solve(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gates: Vec<acir::circuit::Gate>,
    ) -> Result<(), acir::OPCODE> {
        if gates.len() == 0 {
            return Ok(());
        }

        let mut unsolved_gates: Vec<Gate> = Vec::new();

        for gate in gates.into_iter() {
            let unsolved = match &gate {
                Gate::Arithmetic(arith) => {
                    ArithmeticSolver::solve(initial_witness, &arith).is_some()
                }
                Gate::Range(_, _) => {
                    // We do not need to solve for this gate, we have passed responsibility to the underlying
                    // proof system for intermediate witness generation
                    false
                }
                Gate::And(and_gate) => {
                    LogicSolver::solve_and_gate(initial_witness, and_gate);

                    // We compute the result because the other gates may want to use the assignment to generate their assignments
                    false
                }
                Gate::Xor(xor_gate) => {
                    LogicSolver::solve_xor_gate(initial_witness, xor_gate);

                    // We compute the result because the other gates may want to use the assignment to generate their assignments
                    false
                }
                Gate::GadgetCall(gc) => {
                    GadgetCaller::solve_gadget_call(initial_witness, gc)?;

                    false
                }
                Gate::Directive(directive) => match directive {
                    acir::circuit::gate::Directive::Invert { x, result } => {
                        match initial_witness.get(x) {
                            None => true,
                            Some(val) => {
                                let inverse = val.inverse();
                                initial_witness.insert(*result, inverse);
                                false
                            }
                        }
                    }
                },
            };
            if unsolved {
                unsolved_gates.push(gate);
            }
        }

        self.solve(initial_witness, unsolved_gates)
    }
}
