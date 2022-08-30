// Key is currently {NPComplete_lang}_{OptionalFanIn}_ProofSystem_OrgName
// Org name is needed because more than one implementation of the same proof system may arise

pub mod compiler;
pub mod pwg;

use std::collections::BTreeMap;

use acir::{
    circuit::{
        gate::{Directive, GadgetCall},
        Circuit, Gate,
    },
    native_types::{Expression, Witness},
    OPCODE,
};

use crate::pwg::{arithmetic::ArithmeticSolver, logic::LogicSolver};
use num_bigint::BigUint;
use num_traits::One;

// re-export acir
pub use acir;
pub use acir::FieldElement;

pub trait Backend: SmartContract + ProofSystemCompiler + PartialWitnessGenerator {}

/// This component will generate the backend specific output for
/// each OPCODE.
/// Returns an Error if the backend does not support that OPCODE
pub trait PartialWitnessGenerator {
    fn solve(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gates: Vec<Gate>,
    ) -> Result<(), OPCODE> {
        if gates.is_empty() {
            return Ok(());
        }
        let mut unsolved_gates: Vec<Gate> = Vec::new();
        println!("about to enter gates loop");

        for gate in gates.into_iter() {
            let unsolved = match &gate {
                Gate::Arithmetic(arith) => {
                    ArithmeticSolver::solve(initial_witness, arith).is_some()
                }
                // We do not need to solve for this gate, we have passed responsibility to the underlying
                // proof system for intermediate witness generation
                Gate::Range(_, _) => {
                    // We do not need to solve for this gate, we have passed responsibility to the underlying
                    // proof system for intermediate witness generation
                    false
                }
                Gate::And(and_gate) => {
                    !LogicSolver::solve_and_gate(initial_witness, and_gate)
                    // We compute the result because the other gates may want to use the assignment to generate their assignments
                }
                Gate::Xor(xor_gate) => {
                    !LogicSolver::solve_xor_gate(initial_witness, xor_gate)
                    // We compute the result because the other gates may want to use the assignment to generate their assignments
                }
                Gate::GadgetCall(gc) => {
                    Self::solve_gadget_call(initial_witness, gc)?;
                    false
                }
                Gate::Directive(directive) => match directive {
                    Directive::Invert { x, result } => match initial_witness.get(x) {
                        None => true,
                        Some(val) => {
                            let inverse = val.inverse();
                            initial_witness.insert(*result, inverse);
                            false
                        }
                    },

                    Directive::Quotient { a, b, q, r } => {
                        match (
                            Self::get_value(a, initial_witness),
                            Self::get_value(b, initial_witness),
                        ) {
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
                    Directive::Truncate { a, b, c, bit_size } => match initial_witness.get(a) {
                        Some(val_a) => {
                            let pow: BigUint = BigUint::one() << bit_size;

                            let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                            let int_b: BigUint = &int_a % &pow;
                            let int_c: BigUint = (&int_a - &int_b) / &pow;

                            initial_witness.insert(
                                *b,
                                FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()),
                            );
                            initial_witness.insert(
                                *c,
                                FieldElement::from_be_bytes_reduce(&int_c.to_bytes_be()),
                            );
                            false
                        }
                        _ => true,
                    },
                    Directive::Split { a, b, bit_size } => match initial_witness.get(a) {
                        Some(val_a) => {
                            let a_big = BigUint::from_bytes_be(&val_a.to_bytes());
                            for i in 0..*bit_size {
                                let j = i as usize;
                                let v = if a_big.bit(j as u64) {
                                    FieldElement::one()
                                } else {
                                    FieldElement::zero()
                                };
                                initial_witness.insert(b[j], v);
                            }
                            false
                        }
                        _ => true,
                    },
                    Directive::Oddrange { a, b, r, bit_size } => match initial_witness.get(a) {
                        Some(val_a) => {
                            let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                            let pow: BigUint = BigUint::one() << (bit_size - 1);

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
                    },
                },
            };
            if unsolved {
                println!("unsolved gate: {:?}", gate);
                unsolved_gates.push(gate);
            }
        }
        self.solve(initial_witness, unsolved_gates)
    }

    fn solve_gadget_call(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gc: &GadgetCall,
    ) -> Result<(), OPCODE>;

    fn get_value(
        a: &Expression,
        initial_witness: &std::collections::BTreeMap<Witness, FieldElement>,
    ) -> Option<FieldElement> {
        let mut result = a.q_c;
        for i in &a.linear_combinations {
            if let Some(f) = initial_witness.get(&i.1) {
                result += i.0 * *f;
            } else {
                return None;
            }
        }
        for i in &a.mul_terms {
            if let (Some(f), Some(g)) = (initial_witness.get(&i.1), initial_witness.get(&i.2)) {
                result += i.0 * *f * *g;
            } else {
                return None;
            }
        }
        Some(result)
    }
}

pub trait SmartContract {
    // Takes a verification  key and produces a smart contract
    // The platform indicator allows a backend to support multiple smart contract platforms
    //
    // fn verification_key(&self, platform: u8, vk: &[u8]) -> &[u8] {
    //     todo!("currently the backend is not configured to use this.")
    // }

    /// Takes an ACIR circuit, the number of witnesses and the number of public inputs
    /// Then returns an Ethereum smart contract
    ///
    /// XXX: This will be deprecated in future releases for the above method.
    /// This deprecation may happen in two stages:
    /// The first stage will remove `num_witnesses` and `num_public_inputs` parameters.
    /// If we cannot avoid `num_witnesses`, it can be added into the Circuit struct.
    fn eth_contract_from_cs(&self, circuit: Circuit) -> String;
}

pub trait ProofSystemCompiler {
    /// The NPC language that this proof system directly accepts.
    /// It is possible for ACVM to transpile to different languages, however it is advised to create a new backend
    /// as this in most cases will be inefficient. For this reason, we want to throw a hard error
    /// if the language and proof system does not line up.
    fn np_language(&self) -> Language;

    /// Creates a Proof given the circuit description and the witness values.
    /// It is important to note that the intermediate witnesses for blackbox functions will not generated
    /// This is the responsibility of the proof system.
    ///
    /// See `SmartContract` regarding the removal of `num_witnesses` and `num_public_inputs`
    fn prove_with_meta(
        &self,
        circuit: Circuit,
        witness_values: BTreeMap<Witness, FieldElement>,
    ) -> Vec<u8>;

    /// Verifies a Proof, given the circuit description.
    ///
    /// XXX: This will be changed in the future to accept a VerifierKey.
    /// At the moment, the Aztec backend API only accepts a constraint system,
    /// which is why this is here.
    ///
    /// See `SmartContract` regarding the removal of `num_witnesses` and `num_public_inputs`
    fn verify_from_cs(
        &self,
        proof: &[u8],
        public_input: Vec<FieldElement>,
        circuit: Circuit,
    ) -> bool;
}

/// Supported NP complete languages
/// This might need to be in ACIR instead
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}

pub fn hash_constraint_system(cs: &Circuit) {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&format!("{:?}", cs));
    let result = hasher.finalize();
    println!("hash of constraint system : {:x?}", &result[..]);
}
