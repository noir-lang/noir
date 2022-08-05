use acvm::{
    pwg::{arithmetic::ArithmeticSolver, logic::LogicSolver},
    PartialWitnessGenerator,
};
use acvm::acir::{self, circuit::Gate, native_types::Witness};
use acvm::FieldElement;
use num_bigint::BigUint;
use num_traits::One;
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
        if gates.is_empty() {
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
                    acir::circuit::gate::Directive::Quotient { a, b, q, r } => {
                        match (Self::get_value(a, initial_witness), Self::get_value(b, initial_witness)) {
                            (Some(val_a), Some(val_b)) => {
                                let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                                let int_b = BigUint::from_bytes_be(&val_b.to_bytes());
                                let int_r = &int_a % &int_b;
                                let int_q = &int_a / &int_b;
                                initial_witness.insert(
                                    *q,
                                    FieldElement::from_be_bytes_reduce(&int_q.to_bytes_be()),
                                );
                                initial_witness.insert(
                                    *r,
                                    FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()),
                                );
                                false
                            }
                            _ => true,
                        }
                    }
                    acir::circuit::gate::Directive::Truncate { a, b, c, bit_size } => {
                        match initial_witness.get(a) {
                            Some(val_a) => {
                                let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                                let mut pow: BigUint = One::one();
                                pow <<= bit_size;
                                let int_b: BigUint = &int_a % &pow;
                                let int_c: BigUint = (&int_a - &int_b) / &pow;
                                initial_witness.insert(
                                    *b,
                                    FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()),
                                ); //XXX: marlin is using r1cs? if so truncate should be done with split&join as there is probably no rangecheck
                                initial_witness.insert(
                                    *c,
                                    FieldElement::from_be_bytes_reduce(&int_c.to_bytes_be()),
                                );
                                false
                            }
                            _ => true,
                        }
                    }
                    acir::circuit::gate::Directive::Oddrange { a, b, r, bit_size } => {
                        match initial_witness.get(a) {
                            Some(val_a) => {
                                let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                                let mut pow: BigUint = One::one();
                                pow <<= bit_size - 1;
                                let bb = &int_a & &pow;
                                let int_r = &int_a - &bb;
                                let int_b = &bb >> (bit_size - 1);

                                initial_witness.insert(
                                    *b,
                                    FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()),
                                );
                                initial_witness.insert(
                                    *r,
                                    FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()),
                                );
                                false
                            }
                            _ => true,
                        }
                    }
                    acir::circuit::gate::Directive::Split { a, b, bit_size } => { !unimplemented!("split directive not yet implemented for marlin") }
                },
            };
            if unsolved {
                unsolved_gates.push(gate);
            }
        }

        self.solve(initial_witness, unsolved_gates)
    }
}
