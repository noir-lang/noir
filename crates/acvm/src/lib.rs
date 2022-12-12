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
use pwg::binary::BinarySolver;

use crate::pwg::{arithmetic::ArithmeticSolver, logic::LogicSolver};
use num_bigint::BigUint;
use num_traits::{One, Zero};

// re-export acir
pub use acir;
pub use acir::FieldElement;

#[derive(PartialEq, Eq, Debug)]
pub enum GateResolution {
    Resolved,                  //Gate is solved
    Skip,                      //Gate cannot be solved
    UnknownError(String),      //Generic error
    UnsupportedOpcode(OPCODE), //Unsupported Opcode
    UnsatisfiedConstrain,      //Gate is not satisfied
    Solved,                    //Circuit is solved, after a number of passes
}

pub trait Backend: SmartContract + ProofSystemCompiler + PartialWitnessGenerator {}

/// This component will generate the backend specific output for
/// each OPCODE.
/// Returns an Error if the backend does not support that OPCODE
pub trait PartialWitnessGenerator {
    fn solve_gate(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gate: &Gate,
    ) -> GateResolution {
        match gate {
            Gate::Arithmetic(arith) => ArithmeticSolver::solve(initial_witness, arith),
            Gate::Range(w, r) => {
                if let Some(w_value) = initial_witness.get(w) {
                    if w_value.num_bits() > *r {
                        return GateResolution::UnsatisfiedConstrain;
                    }
                    GateResolution::Resolved
                } else {
                    GateResolution::Skip
                }
            }
            Gate::And(and_gate) => {
                LogicSolver::solve_and_gate(initial_witness, and_gate)
                // We compute the result because the other gates may want to use the assignment to generate their assignments
            }
            Gate::Xor(xor_gate) => {
                LogicSolver::solve_xor_gate(initial_witness, xor_gate)
                // We compute the result because the other gates may want to use the assignment to generate their assignments
            }
            Gate::GadgetCall(gc) => {
                let mut unsolvable = false;
                for i in &gc.inputs {
                    if !initial_witness.contains_key(&i.witness) {
                        unsolvable = true;
                        break;
                    }
                }
                if unsolvable {
                    GateResolution::Skip
                } else if let Err(op) = Self::solve_gadget_call(initial_witness, gc) {
                    GateResolution::UnsupportedOpcode(op)
                } else {
                    GateResolution::Resolved
                }
            }
            Gate::Directive(directive) => match directive {
                Directive::Invert { x, result } => match initial_witness.get(x) {
                    None => GateResolution::Skip,
                    Some(val) => {
                        let inverse = val.inverse();
                        initial_witness.insert(*result, inverse);
                        GateResolution::Resolved
                    }
                },
                Directive::Quotient { a, b, q, r, predicate } => {
                    match (Self::get_value(a, initial_witness), Self::get_value(b, initial_witness))
                    {
                        (Some(val_a), Some(val_b)) => {
                            let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                            let int_b = BigUint::from_bytes_be(&val_b.to_bytes());
                            let default = Box::new(Expression::one());
                            let pred = predicate.as_ref().unwrap_or(&default);
                            if let Some(pred_value) = Self::get_value(pred, initial_witness) {
                                let (int_r, int_q) = if pred_value.is_zero() {
                                    (BigUint::zero(), BigUint::zero())
                                } else {
                                    (&int_a % &int_b, &int_a / &int_b)
                                };
                                initial_witness.insert(
                                    *q,
                                    FieldElement::from_be_bytes_reduce(&int_q.to_bytes_be()),
                                );
                                initial_witness.insert(
                                    *r,
                                    FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()),
                                );
                                GateResolution::Resolved
                            } else {
                                GateResolution::Skip
                            }
                        }
                        _ => GateResolution::Skip,
                    }
                }
                Directive::Truncate { a, b, c, bit_size } => match initial_witness.get(a) {
                    Some(val_a) => {
                        let pow: BigUint = BigUint::one() << bit_size;

                        let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                        let int_b: BigUint = &int_a % &pow;
                        let int_c: BigUint = (&int_a - &int_b) / &pow;

                        initial_witness
                            .insert(*b, FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()));
                        initial_witness
                            .insert(*c, FieldElement::from_be_bytes_reduce(&int_c.to_bytes_be()));
                        GateResolution::Resolved
                    }
                    _ => GateResolution::Skip,
                },
                Directive::Split { a, b, bit_size } => match Self::get_value(a, initial_witness) {
                    Some(val_a) => {
                        let a_big = BigUint::from_bytes_be(&val_a.to_bytes());
                        for i in 0..*bit_size {
                            let j = i as usize;
                            let v = if a_big.bit(j as u64) {
                                FieldElement::one()
                            } else {
                                FieldElement::zero()
                            };
                            match initial_witness.entry(b[j]) {
                                std::collections::btree_map::Entry::Vacant(e) => {
                                    e.insert(v);
                                }
                                std::collections::btree_map::Entry::Occupied(e) => {
                                    if e.get() != &v {
                                        return GateResolution::UnsatisfiedConstrain;
                                    }
                                }
                            }
                        }
                        GateResolution::Resolved
                    }
                    _ => GateResolution::Skip,
                },
                Directive::Oddrange { a, b, r, bit_size } => match initial_witness.get(a) {
                    Some(val_a) => {
                        let int_a = BigUint::from_bytes_be(&val_a.to_bytes());
                        let pow: BigUint = BigUint::one() << (bit_size - 1);
                        if int_a >= (&pow << 1) {
                            return GateResolution::UnsatisfiedConstrain;
                        }
                        let bb = &int_a & &pow;
                        let int_r = &int_a - &bb;
                        let int_b = &bb >> (bit_size - 1);

                        initial_witness
                            .insert(*b, FieldElement::from_be_bytes_reduce(&int_b.to_bytes_be()));
                        initial_witness
                            .insert(*r, FieldElement::from_be_bytes_reduce(&int_r.to_bytes_be()));
                        GateResolution::Resolved
                    }
                    _ => GateResolution::Skip,
                },
            },
        }
    }

    fn solve(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        mut gates_to_resolve: Vec<Gate>,
    ) -> GateResolution {
        let mut unresolved_gates: Vec<Gate> = Vec::new();
        let mut ctx = BinarySolver::new();
        //binary_solve is used to manage the binary solving mode:
        //binary_solve.is_none()                => binary solve is not activated
        //binary_solve == Some(forward_pass)    => binary solve is activated and will do forward_pass forward passes, decreasing at each pass until it reach 0 where the process will be backward
        let mut binary_solve = None;

        while !gates_to_resolve.is_empty() {
            unresolved_gates.clear();

            let gates: Box<dyn Iterator<Item = _>> = if binary_solve == Some(0) {
                //we go backward because binary solver should execute only when the program returns an array
                //in that case it is a bit more efficient to go backwards, although both ways work.
                Box::new(gates_to_resolve.iter().rev())
            } else {
                Box::new(gates_to_resolve.iter())
            };
            for gate in gates {
                let mut result = self.solve_gate(initial_witness, gate);
                if binary_solve.is_some() && result == GateResolution::Skip {
                    result = ctx.solve(gate, initial_witness);
                }
                match result {
                    GateResolution::Skip => unresolved_gates.push(gate.clone()),
                    GateResolution::Resolved => (),
                    resolution => return resolution,
                }
            }

            if let Some(forward_pass) = binary_solve {
                if forward_pass > 0 {
                    binary_solve = Some(forward_pass - 1);
                }
            } else if gates_to_resolve.len() == unresolved_gates.len() {
                binary_solve = Some(2);
            }
            std::mem::swap(&mut gates_to_resolve, &mut unresolved_gates);
        }
        GateResolution::Solved
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

    fn get_exact_circuit_size(&self, circuit: Circuit) -> u32;
}

/// Supported NP complete languages
/// This might need to be in ACIR instead
#[derive(Debug, Clone)]
pub enum Language {
    R1CS,
    PLONKCSat { width: usize },
}

pub trait CustomGate {
    fn supports(&self, opcode: &str) -> bool;
    fn supports_gate(&self, gate: &Gate) -> bool;
}

impl CustomGate for Language {
    fn supports(&self, _opcode: &str) -> bool {
        match self {
            Language::R1CS => false,
            Language::PLONKCSat { .. } => true,
        }
    }

    fn supports_gate(&self, gate: &Gate) -> bool {
        !matches!(
            (self, gate),
            (Language::R1CS, Gate::Range(..))
                | (Language::R1CS, Gate::And(..))
                | (Language::R1CS, Gate::Xor(..))
        )
    }
}

pub fn hash_constraint_system(cs: &Circuit) {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&format!("{:?}", cs));
    let result = hasher.finalize();
    println!("hash of constraint system : {:x?}", &result[..]);
}
