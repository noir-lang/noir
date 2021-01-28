use super::crs::CRS;
use super::pippenger::Pippenger;
use super::Barretenberg;
use noir_field::FieldElement as Scalar;
use wasmer_runtime::Value;

pub struct StandardComposer {
    barretenberg: Barretenberg,
    pippenger: Pippenger,
    crs: CRS,
}

impl StandardComposer {
    pub fn new(circuit_size: usize) -> StandardComposer {
        let mut barretenberg = Barretenberg::new();

        let crs = CRS::new(circuit_size);

        let pippenger = Pippenger::new(&crs.g1_data, &mut barretenberg);

        StandardComposer {
            crs,
            barretenberg,
            pippenger,
        }
    }
}

#[derive(Clone)]
pub struct Assignments(Vec<Scalar>);
pub type WitnessAssignments = Assignments;

impl Assignments {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let witness_len = self.0.len() as u32;
        buffer.extend_from_slice(&witness_len.to_be_bytes());

        for assignment in self.0.iter() {
            buffer.extend_from_slice(&assignment.to_bytes());
        }

        buffer
    }

    pub fn push_i32(&mut self, value: i32) {
        self.0.push(Scalar::from(value as i128));
    }
    pub fn push(&mut self, value: Scalar) {
        self.0.push(value);
    }
    pub fn new() -> Assignments {
        Assignments(vec![])
    }
}

#[derive(Clone, Hash, Debug)]
pub struct Constraint {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub qm: Scalar,
    pub ql: Scalar,
    pub qr: Scalar,
    pub qo: Scalar,
    pub qc: Scalar,
}

impl Constraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serialiasing Wires
        buffer.extend_from_slice(&self.a.to_be_bytes());
        buffer.extend_from_slice(&self.b.to_be_bytes());
        buffer.extend_from_slice(&self.c.to_be_bytes());

        // serialise selectors
        buffer.extend_from_slice(&self.qm.to_bytes());
        buffer.extend_from_slice(&self.ql.to_bytes());
        buffer.extend_from_slice(&self.qr.to_bytes());
        buffer.extend_from_slice(&self.qo.to_bytes());
        buffer.extend_from_slice(&self.qc.to_bytes());

        buffer
    }
}

#[derive(Clone, Hash, Debug)]
pub struct RangeConstraint {
    pub a: i32,
    pub num_bits: i32,
}

impl RangeConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serialiasing Wires
        buffer.extend_from_slice(&self.a.to_be_bytes());
        buffer.extend_from_slice(&self.num_bits.to_be_bytes());

        buffer
    }
}
#[derive(Clone, Hash, Debug)]
pub struct MerkleMembershipConstraint {
    pub hash_path: Vec<(i32, i32)>,
    pub root: i32,
    pub leaf: i32,
    pub index: i32,
    pub result: i32,
}

impl MerkleMembershipConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // On the C++ side, it is being deserialized as a single vector
        // So the given length is doubled
        let hash_path_len = (self.hash_path.len()*2) as u32;

        buffer.extend_from_slice(&hash_path_len.to_be_bytes());
        for constraint in self.hash_path.iter() {
            buffer.extend_from_slice(&constraint.0.to_be_bytes());
            buffer.extend_from_slice(&constraint.1.to_be_bytes());
        }

        buffer.extend_from_slice(&self.root.to_be_bytes());
        buffer.extend_from_slice(&self.leaf.to_be_bytes());
        buffer.extend_from_slice(&self.result.to_be_bytes());
        buffer.extend_from_slice(&self.index.to_be_bytes());

        buffer
    }
}
#[derive(Clone, Hash, Debug)]
pub struct MerkleRootConstraint {
    pub leaves: Vec<i32>,
    pub root: i32,
}

impl MerkleRootConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let num_leaves = (self.leaves.len()) as u32;

        buffer.extend_from_slice(&num_leaves.to_be_bytes());
        for leaf in self.leaves.iter() {
            buffer.extend_from_slice(&leaf.to_be_bytes());
        }
        buffer.extend_from_slice(&self.root.to_be_bytes());

        buffer
    }
}
#[derive(Clone, Hash, Debug)]
pub struct Sha256Constraint {
    pub inputs: Vec<(i32, i32)>,
    pub result_low_128: i32,
    pub result_high_128: i32,
}

impl Sha256Constraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let inputs_len = self.inputs.len() as u32;
        buffer.extend_from_slice(&inputs_len.to_be_bytes());
        for constraint in self.inputs.iter() {
            buffer.extend_from_slice(&constraint.0.to_be_bytes());
            buffer.extend_from_slice(&constraint.1.to_be_bytes());
        }

        buffer.extend_from_slice(&self.result_low_128.to_be_bytes());
        buffer.extend_from_slice(&self.result_high_128.to_be_bytes());

        buffer
    }
}

#[derive(Clone, Hash, Debug)]
pub struct LogicConstraint {
    a: i32,
    b: i32,
    result: i32,
    num_bits: i32,
    is_xor_gate: bool,
}

impl LogicConstraint {
    pub fn and(a: i32, b: i32, result: i32, num_bits: i32) -> LogicConstraint {
        LogicConstraint {
            a,
            b,
            result,
            num_bits,
            is_xor_gate: false,
        }
    }
    pub fn xor(a: i32, b: i32, result: i32, num_bits: i32) -> LogicConstraint {
        LogicConstraint {
            a,
            b,
            result,
            num_bits,
            is_xor_gate: true,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serialiasing Wires
        buffer.extend_from_slice(&self.a.to_be_bytes());
        buffer.extend_from_slice(&self.b.to_be_bytes());
        buffer.extend_from_slice(&self.result.to_be_bytes());
        buffer.extend_from_slice(&self.num_bits.to_be_bytes());
        buffer.extend_from_slice(&i32::to_be_bytes(self.is_xor_gate as i32));

        buffer
    }
}

#[derive(Clone, Hash, Debug)]
pub struct ConstraintSystem {
    pub var_num: u32,
    pub pub_var_num: u32,

    pub logic_constraints: Vec<LogicConstraint>,
    pub range_constraints: Vec<RangeConstraint>,
    pub sha256_constraints: Vec<Sha256Constraint>,
    pub merkle_membership_constraints: Vec<MerkleMembershipConstraint>,
    pub merkle_root_constraints: Vec<MerkleRootConstraint>,
    pub constraints: Vec<Constraint>,
}

impl ConstraintSystem {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // Push lengths onto the buffer
        buffer.extend_from_slice(&self.var_num.to_be_bytes());
        buffer.extend_from_slice(&self.pub_var_num.to_be_bytes());

        // Serialise each Logic constraint
        let logic_constraints_len = self.logic_constraints.len() as u32;
        buffer.extend_from_slice(&logic_constraints_len.to_be_bytes());
        for constraint in self.logic_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each Range constraint
        let range_constraints_len = self.range_constraints.len() as u32;
        buffer.extend_from_slice(&range_constraints_len.to_be_bytes());
        for constraint in self.range_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each Sha256 constraint
        let sha256_constraints_len = self.sha256_constraints.len() as u32;
        buffer.extend_from_slice(&sha256_constraints_len.to_be_bytes());
        for constraint in self.sha256_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each Merkle Membership constraint
        let merkle_membership_constraints_len = self.merkle_membership_constraints.len() as u32;
        buffer.extend_from_slice(&merkle_membership_constraints_len.to_be_bytes());
        for constraint in self.merkle_membership_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each Merkle Root constraint
        let merkle_root_len = self.merkle_root_constraints.len() as u32;
        buffer.extend_from_slice(&merkle_root_len.to_be_bytes());
        for constraint in self.merkle_root_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }
        
        // Serialise each Arithmetic constraint
        let constraints_len = self.constraints.len() as u32;
        buffer.extend_from_slice(&constraints_len.to_be_bytes());
        for constraint in self.constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        buffer
    }

    pub fn size(&self) -> usize {
        // XXX: We do not want the user to need to enter the circuit size for a circuit
        // as this is prone to error. For now, we will create a dummy standard composer, which will
        // call get_circuit_size and then we drop it
        let mut dummy_composer = StandardComposer::new(2);
        dummy_composer.get_circuit_size(&self) as usize
    }
}

impl StandardComposer {

    // XXX: This does not belong here. Ideally, the Rust code should generate the SC code
    // Since it's already done in C++, we are just re-exporting for now
    pub fn smart_contract(&mut self, constraint_system: &ConstraintSystem) -> String {
        use std::convert::TryInto;
        let cs_buf = constraint_system.to_bytes();
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let g2_ptr = self.barretenberg.allocate(&self.crs.g2_data);

        let contract_size = self.barretenberg
        .call_multiple("composer__smart_contract", vec![    
                &self.pippenger.pointer(),&g2_ptr,&cs_ptr,&Value::I32(0)
        ])
        .value();
        let contract_ptr = self.barretenberg.slice_memory(0, 4);
        let contract_ptr = u32::from_le_bytes(contract_ptr[0..4].try_into().unwrap());

        let sc_as_bytes = self.barretenberg.slice_memory(
            contract_ptr as usize,
            contract_ptr as usize + contract_size.to_u128() as usize,
        );

        // XXX: We truncate the first 40 bytes, due to it being mangled
        // For some reason, the first line is partially mangled
        // So in C+ the first line is duplicated and then truncated
        let verification_method : String = sc_as_bytes[40..].iter().map(|b| *b as char).collect();
        crate::contract::turbo_verifier::create(&verification_method)

    }

    pub fn get_circuit_size(&mut self, constraint_system: &ConstraintSystem) -> u128 {
        let cs_buf = constraint_system.to_bytes();
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let circuit_size = self
            .barretenberg
            .call("composer__get_circuit_size", &cs_ptr)
            .value();

        self.barretenberg.free(cs_ptr);

        circuit_size.to_u128()
    }

    pub fn create_proof(
        &mut self,
        constraint_system: &ConstraintSystem,
        witness: WitnessAssignments,
    ) -> Vec<u8> {    
        use core::convert::TryInto;


        let cs_buf = constraint_system.to_bytes();
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let witness_buf = witness.to_bytes();
        let witness_ptr = self.barretenberg.allocate(&witness_buf);

        let g2_ptr = self.barretenberg.allocate(&self.crs.g2_data);

        let proof_size = self
            .barretenberg
            .call_multiple(
                "composer__new_proof",
                vec![&self.pippenger.pointer(), &g2_ptr, &cs_ptr, &witness_ptr, &Value::I32(0)],
            )
            .value();

        let proof_ptr = self.barretenberg.slice_memory(0, 4);
        let proof_ptr = u32::from_le_bytes(proof_ptr[0..4].try_into().unwrap());

        let proof =  self.barretenberg.slice_memory(
            proof_ptr as usize,
            proof_ptr as usize + proof_size.to_u128() as usize,
        );

        return proof
    }

    pub fn verify(
        &mut self,
        constraint_system: &ConstraintSystem,
        proof: &[u8],
        public_inputs: Option<Assignments>,
    ) -> bool {
        let cs_buf = constraint_system.to_bytes();
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let proof_ptr = self.barretenberg.allocate(proof);

        let g2_ptr = self.barretenberg.allocate(&self.crs.g2_data);

        let verified = match public_inputs {
            None => {
                let verified = self
                    .barretenberg
                    .call_multiple(
                        "composer__verify_proof",
                        vec![
                            &self.pippenger.pointer(),
                            &g2_ptr,
                            &cs_ptr,
                            &proof_ptr,
                            &Value::I32(proof.len() as i32),
                        ],
                    )
                    .value();
                verified.clone()
            }
            Some(pub_inputs) => {
                let pub_inputs_buf = pub_inputs.to_bytes();
                let pub_inputs_ptr = self.barretenberg.allocate(&pub_inputs_buf);

                let verified = self
                    .barretenberg
                    .call_multiple(
                        "composer__verify_proof_with_public_inputs",
                        vec![
                            &self.pippenger.pointer(),
                            &g2_ptr,
                            &cs_ptr,
                            &pub_inputs_ptr,
                            &proof_ptr,
                            &Value::I32(proof.len() as i32),
                        ],
                    )
                    .value();

                self.barretenberg.free(pub_inputs_ptr);

                verified.clone()
            }
        };
        // self.barretenberg.free(cs_ptr);
        self.barretenberg.free(proof_ptr);
        // self.barretenberg.free(g2_ptr);

        match verified.to_u128() {
            0 => false,
            1 => true,
            _ => panic!("Expected a 1 or a zero for the verification result"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_a_single_constraint() {
        let constraint = Constraint {
            a: 1,
            b: 2,
            c: 3,
            qm: Scalar::zero(),
            ql: Scalar::one(),
            qr: Scalar::one(),
            qo: -Scalar::one(),
            qc: Scalar::zero(),
        };

        let constraint_system = ConstraintSystem {
            var_num: 3,
            pub_var_num: 0,
            logic_constraints: vec![],
            range_constraints: vec![],
            sha256_constraints: vec![],
            merkle_membership_constraints: vec![],
            merkle_root_constraints : vec![],
            constraints: vec![constraint.clone()],
        };

        let case_1 = WitnessResult {
            witness: Assignments(vec![(-1).into(), 2.into(), 1.into()]),
            public_inputs: None,
            result: true,
        };
        let case_2 = WitnessResult {
            witness: Assignments(vec![Scalar::zero(), Scalar::zero(), Scalar::zero()]),
            public_inputs: None,
            result: true,
        };
        let case_3 = WitnessResult {
            witness: Assignments(vec![10.into(), (-3).into(), 7.into()]),
            public_inputs: None,
            result: true,
        };
        let case_4 = WitnessResult {
            witness: Assignments(vec![Scalar::zero(), Scalar::zero(), Scalar::one()]),
            public_inputs: None,
            result: false,
        };
        let case_5 = WitnessResult {
            witness: Assignments(vec![Scalar::one(), 2.into(), 6.into()]),
            public_inputs: None,
            result: false,
        };
        // This should fail as we specified that we do not have any public inputs
        let case_6a = WitnessResult {
            witness: Assignments(vec![Scalar::one(), 2.into(), 6.into()]),
            public_inputs: Some(Assignments(vec![Scalar::one()])),
            result: false,
        };
        // Even if the public input is zero
        let case_6b = WitnessResult {
            witness: Assignments(vec![Scalar::one(), Scalar::from(2), Scalar::from(6)]),
            public_inputs: Some(Assignments(vec![Scalar::zero()])),
            result: false,
        };

        test_circuit(
            &constraint_system,
            vec![
                case_1.clone(),
                case_2.clone(),
                case_3.clone(),
                case_4.clone(),
                case_5.clone(),
                case_6a,
                case_6b,
            ],
        );

        // Now lets create the same constraint system
        // However, we specify that we want to reserve space for 2 public inputs.
        // Test cases should work the same, even though we supply no public inputs
        // It should also work, if we supply the correct public inputs
        let constraint_system = ConstraintSystem {
            var_num: 3,
            pub_var_num: 2,
            logic_constraints: vec![],
            range_constraints: vec![],
            sha256_constraints: vec![],
            merkle_membership_constraints: vec![],
            merkle_root_constraints : vec![],
            constraints: vec![constraint],
        };

        let case_7 = WitnessResult {
            witness: Assignments(vec![Scalar::one(), 2.into(), 3.into()]),
            public_inputs: Some(Assignments(vec![1.into(), 2.into()])),
            result: true,
        };
        let case_8 = WitnessResult {
            witness: Assignments(vec![Scalar::one(), 2.into(), 3.into()]),
            public_inputs: Some(Assignments(vec![Scalar::one(), 3.into()])),
            result: false,
        };

        test_circuit(
            &constraint_system,
            vec![case_1, case_2, case_3, case_4, case_5, case_7, case_8],
        );
    }

    #[test]
    fn test_multiple_constraints() {
        let constraint = Constraint {
            a: 1,
            b: 2,
            c: 3,
            qm: Scalar::zero(),
            ql: Scalar::one(),
            qr: Scalar::one(),
            qo: -Scalar::one(),
            qc: Scalar::zero(),
        };
        let constraint2 = Constraint {
            a: 2,
            b: 3,
            c: 4,
            qm: Scalar::one(),
            ql: Scalar::zero(),
            qr: Scalar::zero(),
            qo: -Scalar::one(),
            qc: Scalar::one(),
        };

        let constraint_system = ConstraintSystem {
            var_num: 4,
            pub_var_num: 1,
            logic_constraints: vec![],
            range_constraints: vec![],
            sha256_constraints: vec![],
            merkle_membership_constraints: vec![],
            merkle_root_constraints : vec![],
            constraints: vec![constraint, constraint2],
        };

        let case_1 = WitnessResult {
            witness: Assignments(vec![1.into(), 1.into(), 2.into(), 3.into()]),
            public_inputs: None,
            result: true,
        };
        let case_2 = WitnessResult {
            witness: Assignments(vec![1.into(), 1.into(), 2.into(), 13.into()]),
            public_inputs: None,
            result: false,
        };

        test_circuit(&constraint_system, vec![case_1, case_2]);
    }

    #[derive(Clone)]
    struct WitnessResult {
        witness: WitnessAssignments,
        public_inputs: Option<Assignments>,
        result: bool,
    }

    fn test_circuit(constraint_system: &ConstraintSystem, test_cases: Vec<WitnessResult>) {
        let mut sc = StandardComposer::new(constraint_system.size());

        for test_case in test_cases.into_iter() {
            let proof = sc.create_proof(&constraint_system, test_case.witness);
            let verified = sc.verify(&constraint_system, &proof, test_case.public_inputs);
            assert_eq!(verified, test_case.result);
        }
    }
}
