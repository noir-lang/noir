/// Partial Witness Generator
/// This is the default partial witness generator for ACIR
mod arithmetic;
mod gadget_call;
mod logic;
pub mod merkle;

use acvm::acir::circuit::Gate;
use acvm::acir::native_types::Witness;
pub use arithmetic::ArithmeticSolver;
pub use gadget_call::GadgetCaller;
pub use logic::LogicSolver;
use noir_field::FieldElement;
use std::collections::BTreeMap;

// XXX: This can be changed considerably, if the Aztec backend could be made "interactive", so we could create witnesses on the fly and add variables on the fly
// If that was the case, we would simply call a composer.sha256() method to compute the necessary values
// We could get rid of the serialiser altogether furthermore, and simply implement a Solver in the aztec_backend
// I further think, that we could get the aztec_backend to return the output values as field elements, so that we can add it into the witness map

pub struct Solver;

impl Solver {
    /// Derives most of the witness based on the initial low level variables
    pub fn solve(initial_witness: &mut BTreeMap<Witness, FieldElement>, gates: Vec<Gate>) {
        if gates.len() == 0 {
            return;
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
                    GadgetCaller::solve_gadget_call(initial_witness, gc);

                    false
                }
            };
            if unsolved {
                unsolved_gates.push(gate);
            }
        }
        Solver::solve(initial_witness, unsolved_gates)
    }
}
