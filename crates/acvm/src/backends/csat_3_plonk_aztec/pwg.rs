use std::collections::BTreeMap;

use acir::{circuit::Gate, native_types::Witness};
use noir_field::FieldElement;

use crate::{
    pwg::{arithmetic::ArithmeticSolver, logic::LogicSolver},
    PartialWitnessGenerator,
};
mod gadget_call;
mod merkle;

use self::gadget_call::GadgetCaller;

use super::Plonk;

impl PartialWitnessGenerator for Plonk {
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
            };
            if unsolved {
                unsolved_gates.push(gate);
            }
        }
        self.solve(initial_witness, unsolved_gates)
    }
}
