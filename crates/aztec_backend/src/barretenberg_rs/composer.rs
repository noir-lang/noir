use super::crs::CRS;
use super::pippenger::Pippenger;
use super::Barretenberg;
use noir_field::FieldElement as Scalar;
use wasmer::Value;

pub struct StandardComposer {
    barretenberg: Barretenberg,
    pippenger: Pippenger,
    crs: CRS,
    constraint_system: ConstraintSystem,
}

impl StandardComposer {
    pub fn new(constraint_system: ConstraintSystem) -> StandardComposer {
        let mut barretenberg = Barretenberg::new();

        let circuit_size =
            StandardComposer::get_circuit_size(&mut barretenberg, &constraint_system);

        let crs = CRS::new(circuit_size as usize);

        let pippenger = Pippenger::new(&crs.g1_data, &mut barretenberg);

        StandardComposer {
            barretenberg,
            pippenger,
            crs,
            constraint_system,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Assignments(pub(crate) Vec<Scalar>);
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

    pub fn from_vec(vec: Vec<Scalar>) -> Assignments {
        Assignments(vec)
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

impl Default for Assignments {
    fn default() -> Self {
        Self::new()
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
        // serialise Wires
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
pub struct EcdsaConstraint {
    pub hashed_message: Vec<i32>,
    pub signature: [i32; 64],
    pub public_key_x: [i32; 32],
    pub public_key_y: [i32; 32],
    pub result: i32,
}

impl EcdsaConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let message_len = (self.hashed_message.len()) as u32;
        buffer.extend_from_slice(&message_len.to_be_bytes());
        for constraint in self.hashed_message.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        let sig_len = (self.signature.len()) as u32;
        buffer.extend_from_slice(&sig_len.to_be_bytes());
        for sig_byte in self.signature.iter() {
            buffer.extend_from_slice(&sig_byte.to_be_bytes());
        }

        let pub_key_x_len = (self.public_key_x.len()) as u32;
        buffer.extend_from_slice(&pub_key_x_len.to_be_bytes());
        for x_byte in self.public_key_x.iter() {
            buffer.extend_from_slice(&x_byte.to_be_bytes());
        }

        let pub_key_y_len = (self.public_key_y.len()) as u32;
        buffer.extend_from_slice(&pub_key_y_len.to_be_bytes());
        for y_byte in self.public_key_y.iter() {
            buffer.extend_from_slice(&y_byte.to_be_bytes());
        }

        buffer.extend_from_slice(&self.result.to_be_bytes());

        buffer
    }
}
#[derive(Clone, Hash, Debug)]
pub struct SchnorrConstraint {
    pub message: Vec<i32>,
    pub signature: [i32; 64],
    pub public_key_x: i32,
    pub public_key_y: i32,
    pub result: i32,
}

impl SchnorrConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let message_len = (self.message.len()) as u32;
        buffer.extend_from_slice(&message_len.to_be_bytes());
        for constraint in self.message.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        let sig_len = (self.signature.len()) as u32;
        buffer.extend_from_slice(&sig_len.to_be_bytes());
        for sig_byte in self.signature.iter() {
            buffer.extend_from_slice(&sig_byte.to_be_bytes());
        }

        buffer.extend_from_slice(&self.public_key_x.to_be_bytes());
        buffer.extend_from_slice(&self.public_key_y.to_be_bytes());
        buffer.extend_from_slice(&self.result.to_be_bytes());

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
        let hash_path_len = (self.hash_path.len() * 2) as u32;

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
// This is the same as MerkleCheckMembership
#[derive(Clone, Hash, Debug)]
pub struct InsertMerkleConstraint {
    pub hash_path: Vec<(i32, i32)>,
    pub root: i32,
    pub leaf: i32,
    pub index: i32,
    pub result: i32,
}

impl InsertMerkleConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // On the C++ side, it is being deserialized as a single vector
        // So the given length is doubled
        let hash_path_len = (self.hash_path.len() * 2) as u32;

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
pub struct Sha256Constraint {
    pub inputs: Vec<(i32, i32)>,
    pub result: [i32; 32],
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

        let result_len = self.result.len() as u32;
        buffer.extend_from_slice(&result_len.to_be_bytes());
        for constraint in self.result.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        buffer
    }
}
#[derive(Clone, Hash, Debug)]
pub struct Blake2sConstraint {
    pub inputs: Vec<(i32, i32)>,
    pub result: [i32; 32],
}

impl Blake2sConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let inputs_len = self.inputs.len() as u32;
        buffer.extend_from_slice(&inputs_len.to_be_bytes());
        for constraint in self.inputs.iter() {
            buffer.extend_from_slice(&constraint.0.to_be_bytes());
            buffer.extend_from_slice(&constraint.1.to_be_bytes());
        }

        let result_len = self.result.len() as u32;
        buffer.extend_from_slice(&result_len.to_be_bytes());
        for constraint in self.result.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        buffer
    }
}
#[derive(Clone, Hash, Debug)]
pub struct HashToFieldConstraint {
    pub inputs: Vec<(i32, i32)>,
    pub result: i32,
}

impl HashToFieldConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let inputs_len = self.inputs.len() as u32;
        buffer.extend_from_slice(&inputs_len.to_be_bytes());
        for constraint in self.inputs.iter() {
            buffer.extend_from_slice(&constraint.0.to_be_bytes());
            buffer.extend_from_slice(&constraint.1.to_be_bytes());
        }

        buffer.extend_from_slice(&self.result.to_be_bytes());

        buffer
    }
}
#[derive(Clone, Hash, Debug)]
pub struct PedersenConstraint {
    pub inputs: Vec<i32>,
    pub result_x: i32,
    pub result_y: i32,
}

impl PedersenConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let inputs_len = self.inputs.len() as u32;
        buffer.extend_from_slice(&inputs_len.to_be_bytes());
        for constraint in self.inputs.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        buffer.extend_from_slice(&self.result_x.to_be_bytes());
        buffer.extend_from_slice(&self.result_y.to_be_bytes());

        buffer
    }
}
#[derive(Clone, Hash, Debug)]
pub struct FixedBaseScalarMulConstraint {
    pub scalar: i32,
    pub pubkey_x: i32,
    pub pubkey_y: i32,
}

impl FixedBaseScalarMulConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serialising Wires
        buffer.extend_from_slice(&self.scalar.to_be_bytes());
        buffer.extend_from_slice(&self.pubkey_x.to_be_bytes());
        buffer.extend_from_slice(&self.pubkey_y.to_be_bytes());

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
        // Serialising Wires
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
    pub public_inputs: Vec<u32>,

    pub logic_constraints: Vec<LogicConstraint>,
    pub range_constraints: Vec<RangeConstraint>,
    pub sha256_constraints: Vec<Sha256Constraint>,
    pub merkle_membership_constraints: Vec<MerkleMembershipConstraint>,
    pub insert_merkle_constraints: Vec<InsertMerkleConstraint>,
    pub schnorr_constraints: Vec<SchnorrConstraint>,
    pub ecdsa_secp256k1_constraints: Vec<EcdsaConstraint>,
    pub blake2s_constraints: Vec<Blake2sConstraint>,
    pub pedersen_constraints: Vec<PedersenConstraint>,
    pub hash_to_field_constraints: Vec<HashToFieldConstraint>,
    pub fixed_base_scalar_mul_constraints: Vec<FixedBaseScalarMulConstraint>,
    pub constraints: Vec<Constraint>,
}

impl ConstraintSystem {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // Push lengths onto the buffer
        buffer.extend_from_slice(&self.var_num.to_be_bytes());

        let pi_len = self.public_inputs.len() as u32;
        buffer.extend_from_slice(&pi_len.to_be_bytes());
        for pub_input in self.public_inputs.iter() {
            buffer.extend_from_slice(&pub_input.to_be_bytes());
        }

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

        // Serialise each Insert Merkle constraint
        let insert_merkle_constraints_len = self.insert_merkle_constraints.len() as u32;
        buffer.extend_from_slice(&insert_merkle_constraints_len.to_be_bytes());
        for constraint in self.insert_merkle_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each Schnorr constraint
        let schnorr_len = self.schnorr_constraints.len() as u32;
        buffer.extend_from_slice(&schnorr_len.to_be_bytes());
        for constraint in self.schnorr_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each ECDSA constraint
        let ecdsa_len = self.ecdsa_secp256k1_constraints.len() as u32;
        buffer.extend_from_slice(&ecdsa_len.to_be_bytes());
        for constraint in self.ecdsa_secp256k1_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each Blake2s constraint
        let blake2s_len = self.blake2s_constraints.len() as u32;
        buffer.extend_from_slice(&blake2s_len.to_be_bytes());
        for constraint in self.blake2s_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each Pedersen constraint
        let pedersen_len = self.pedersen_constraints.len() as u32;
        buffer.extend_from_slice(&pedersen_len.to_be_bytes());
        for constraint in self.pedersen_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each HashToField constraint
        let h2f_len = self.hash_to_field_constraints.len() as u32;
        buffer.extend_from_slice(&h2f_len.to_be_bytes());
        for constraint in self.hash_to_field_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialise each HashToField constraint
        let fixed_base_scalar_mul_len = self.fixed_base_scalar_mul_constraints.len() as u32;
        buffer.extend_from_slice(&fixed_base_scalar_mul_len.to_be_bytes());
        for constraint in self.fixed_base_scalar_mul_constraints.iter() {
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
}

impl StandardComposer {
    // XXX: This does not belong here. Ideally, the Rust code should generate the SC code
    // Since it's already done in C++, we are just re-exporting for now
    pub fn smart_contract(&mut self) -> String {
        use std::convert::TryInto;
        let cs_buf = self.constraint_system.to_bytes();
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let g2_ptr = self.barretenberg.allocate(&self.crs.g2_data);

        let contract_size = self
            .barretenberg
            .call_multiple(
                "composer__smart_contract",
                vec![&self.pippenger.pointer(), &g2_ptr, &cs_ptr, &Value::I32(0)],
            )
            .value();
        let contract_ptr = self.barretenberg.slice_memory(0, 4);
        let contract_ptr = u32::from_le_bytes(contract_ptr[0..4].try_into().unwrap());

        let sc_as_bytes = self.barretenberg.slice_memory(
            contract_ptr as usize,
            contract_ptr as usize + contract_size.unwrap_i32() as usize,
        );

        // XXX: We truncate the first 40 bytes, due to it being mangled
        // For some reason, the first line is partially mangled
        // So in C+ the first line is duplicated and then truncated
        let verification_method: String = sc_as_bytes[40..].iter().map(|b| *b as char).collect();
        crate::contract::turbo_verifier::create(&verification_method)
    }

    // XXX: There seems to be a bug in the C++ code
    // where it causes a `HeapAccessOutOfBound` error
    // for certain circuit sizes.
    //
    // This method calls the WASM for the circuit size
    // if an error is returned, then the circuit size is defaulted to 2^19.
    //
    // This method is primarily used to determine how many group
    // elements we need from the CRS. So using 2^19 on an error
    // should be an overestimation.
    pub fn get_circuit_size(
        barretenberg: &mut Barretenberg,
        constraint_system: &ConstraintSystem,
    ) -> u32 {
        let cs_buf = constraint_system.to_bytes();
        let cs_ptr = barretenberg.allocate(&cs_buf);

        let func = barretenberg
            .instance
            .exports
            .get_function("composer__get_circuit_size")
            .unwrap();

        let params: Vec<_> = vec![cs_ptr.clone()];
        match func.call(&params) {
            Ok(vals) => {
                let i32_bytes = vals.first().cloned().unwrap().unwrap_i32().to_be_bytes();
                let u32_val = u32::from_be_bytes(i32_bytes);
                barretenberg.free(cs_ptr);
                u32_val
            }
            Err(_) => {
                // Default to 2^19
                2u32.pow(19)
            }
        }
    }

    pub fn create_proof(&mut self, witness: WitnessAssignments) -> Vec<u8> {
        use core::convert::TryInto;
        let now = std::time::Instant::now();

        let cs_buf = self.constraint_system.to_bytes();
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let witness_buf = witness.to_bytes();
        let witness_ptr = self.barretenberg.allocate(&witness_buf);

        let g2_ptr = self.barretenberg.allocate(&self.crs.g2_data);

        let proof_size = self
            .barretenberg
            .call_multiple(
                "composer__new_proof",
                vec![
                    &self.pippenger.pointer(),
                    &g2_ptr,
                    &cs_ptr,
                    &witness_ptr,
                    &Value::I32(0),
                ],
            )
            .value();

        let proof_ptr = self.barretenberg.slice_memory(0, 4);
        let proof_ptr = u32::from_le_bytes(proof_ptr[0..4].try_into().unwrap());

        let proof = self.barretenberg.slice_memory(
            proof_ptr as usize,
            proof_ptr as usize + proof_size.unwrap_i32() as usize,
        );
        println!(
            "Total Proving time (Rust + WASM) : {}ns ~ {}seconds",
            now.elapsed().as_nanos(),
            now.elapsed().as_secs(),
        );
        remove_public_inputs(self.constraint_system.public_inputs.len(), proof)
    }

    pub fn verify(
        &mut self,
        // XXX: Important: This assumes that the proof does not have the public inputs pre-pended to it
        // This is not the case, if you take the proof directly from Barretenberg
        proof: &[u8],
        public_inputs: Option<Assignments>,
    ) -> bool {
        // Prepend the public inputs to the proof.
        // This is how Barretenberg expects it to be.
        // This is non-standard however, so this Rust wrapper will strip the public inputs
        // from proofs created by Barretenberg. Then in Verify we prepend them again.
        //

        let mut proof = proof.to_vec();
        if let Some(pi) = &public_inputs {
            let mut proof_with_pi = Vec::new();
            for assignment in pi.0.iter() {
                proof_with_pi.extend(&assignment.to_bytes());
            }
            proof_with_pi.extend(proof);
            proof = proof_with_pi;
        }
        let now = std::time::Instant::now();

        let cs_buf = self.constraint_system.to_bytes();
        let cs_ptr = self.barretenberg.allocate(&cs_buf);

        let proof_ptr = self.barretenberg.allocate(&proof);

        let g2_ptr = self.barretenberg.allocate(&self.crs.g2_data);

        let verified = match public_inputs {
            None => self
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
                .value(),
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

                verified
            }
        };
        // self.barretenberg.free(cs_ptr);
        self.barretenberg.free(proof_ptr);
        // self.barretenberg.free(g2_ptr);

        println!(
            "Total Verifier time (Rust + WASM) : {}ns ~ {}seconds",
            now.elapsed().as_nanos(),
            now.elapsed().as_secs(),
        );

        match verified.unwrap_i32() {
            0 => false,
            1 => true,
            _ => panic!("Expected a 1 or a zero for the verification result"),
        }
    }
}

fn remove_public_inputs(num_pub_inputs: usize, proof: Vec<u8>) -> Vec<u8> {
    // This is only for public inputs and for Barretenberg.
    // Barretenberg only used bn254, so each element is 32 bytes.
    // To remove the public inputs, we need to remove (num_pub_inputs * 32) bytes
    let num_bytes_to_remove = 32 * num_pub_inputs;
    proof[num_bytes_to_remove..].to_vec()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_a_single_constraint_no_pub_inputs() {
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
            var_num: 4,
            public_inputs: vec![],
            logic_constraints: vec![],
            range_constraints: vec![],
            sha256_constraints: vec![],
            merkle_membership_constraints: vec![],
            schnorr_constraints: vec![],
            blake2s_constraints: vec![],
            pedersen_constraints: vec![],
            hash_to_field_constraints: vec![],
            constraints: vec![constraint],
            ecdsa_secp256k1_constraints: vec![],
            fixed_base_scalar_mul_constraints: vec![],
            insert_merkle_constraints: vec![],
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
            constraint_system,
            vec![case_1, case_2, case_3, case_4, case_5, case_6a, case_6b],
        );
    }
    #[test]
    fn test_a_single_constraint_with_pub_inputs() {
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
            var_num: 4,
            public_inputs: vec![1, 2],
            logic_constraints: vec![],
            range_constraints: vec![],
            sha256_constraints: vec![],
            merkle_membership_constraints: vec![],
            schnorr_constraints: vec![],
            blake2s_constraints: vec![],
            pedersen_constraints: vec![],
            hash_to_field_constraints: vec![],
            constraints: vec![constraint],
            ecdsa_secp256k1_constraints: vec![],
            fixed_base_scalar_mul_constraints: vec![],
            insert_merkle_constraints: vec![],
        };

        // This fails because the constraint system requires public inputs,
        // but none are supplied in public_inputs. So the verifier will not
        // supply anything.
        let case_1 = WitnessResult {
            witness: Assignments(vec![(-1).into(), 2.into(), 1.into()]),
            public_inputs: None,
            result: false,
        };
        let case_2 = WitnessResult {
            witness: Assignments(vec![Scalar::zero(), Scalar::zero(), Scalar::zero()]),
            public_inputs: Some(Assignments(vec![Scalar::zero(), Scalar::zero()])),
            result: true,
        };

        let case_3 = WitnessResult {
            witness: Assignments(vec![Scalar::one(), 2.into(), 6.into()]),
            public_inputs: Some(Assignments(vec![Scalar::one(), 3.into()])),
            result: false,
        };

        // Not enough public inputs
        let case_4 = WitnessResult {
            witness: Assignments(vec![Scalar::one(), Scalar::from(2), Scalar::from(6)]),
            public_inputs: Some(Assignments(vec![Scalar::one()])),
            result: false,
        };

        let case_5 = WitnessResult {
            witness: Assignments(vec![Scalar::one(), 2.into(), 3.into()]),
            public_inputs: Some(Assignments(vec![Scalar::one(), 2.into()])),
            result: true,
        };

        let case_6 = WitnessResult {
            witness: Assignments(vec![Scalar::one(), 2.into(), 3.into()]),
            public_inputs: Some(Assignments(vec![Scalar::one(), 3.into()])),
            result: false,
        };

        test_circuit(
            constraint_system,
            vec![case_1, case_2, case_3, case_4, case_5, case_6],
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
            var_num: 5,
            public_inputs: vec![1],
            logic_constraints: vec![],
            range_constraints: vec![],
            sha256_constraints: vec![],
            merkle_membership_constraints: vec![],
            schnorr_constraints: vec![],
            blake2s_constraints: vec![],
            pedersen_constraints: vec![],
            hash_to_field_constraints: vec![],
            constraints: vec![constraint, constraint2],
            ecdsa_secp256k1_constraints: vec![],
            fixed_base_scalar_mul_constraints: vec![],
            insert_merkle_constraints: vec![],
        };

        let case_1 = WitnessResult {
            witness: Assignments(vec![1.into(), 1.into(), 2.into(), 3.into()]),
            public_inputs: Some(Assignments(vec![Scalar::one()])),
            result: true,
        };
        let case_2 = WitnessResult {
            witness: Assignments(vec![1.into(), 1.into(), 2.into(), 13.into()]),
            public_inputs: Some(Assignments(vec![Scalar::one()])),
            result: false,
        };

        test_circuit(constraint_system, vec![case_1, case_2]);
    }
    #[test]
    fn test_schnorr_constraints() {
        let mut signature_indices = [0i32; 64];
        for i in 13..(13 + 64) {
            signature_indices[i - 13] = i as i32;
        }
        let result_indice = signature_indices.last().unwrap() + 1;

        let constraint = SchnorrConstraint {
            message: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            public_key_x: 11,
            public_key_y: 12,
            signature: signature_indices,
            result: result_indice,
        };

        let arith_constraint = Constraint {
            a: result_indice,
            b: result_indice,
            c: result_indice,
            qm: Scalar::zero(),
            ql: Scalar::zero(),
            qr: Scalar::zero(),
            qo: Scalar::one(),
            qc: -Scalar::one(),
        };

        let constraint_system = ConstraintSystem {
            var_num: 80,
            public_inputs: vec![],
            logic_constraints: vec![],
            range_constraints: vec![],
            sha256_constraints: vec![],
            merkle_membership_constraints: vec![],
            schnorr_constraints: vec![constraint],
            blake2s_constraints: vec![],
            pedersen_constraints: vec![],
            hash_to_field_constraints: vec![],
            constraints: vec![arith_constraint],
            ecdsa_secp256k1_constraints: vec![],
            fixed_base_scalar_mul_constraints: vec![],
            insert_merkle_constraints: vec![],
        };

        let pub_x =
            Scalar::from_hex("0x17cbd3ed3151ccfd170efe1d54280a6a4822640bf5c369908ad74ea21518a9c5")
                .unwrap();
        let pub_y =
            Scalar::from_hex("0x0e0456e3795c1a31f20035b741cd6158929eeccd320d299cfcac962865a6bc74")
                .unwrap();

        let sig: [i128; 64] = [
            7, 131, 147, 205, 145, 77, 60, 169, 159, 86, 91, 209, 140, 210, 4, 21, 186, 39, 221,
            195, 62, 35, 220, 144, 135, 28, 201, 97, 145, 125, 146, 211, 92, 16, 67, 59, 162, 133,
            144, 52, 184, 137, 241, 102, 176, 152, 138, 220, 21, 40, 211, 178, 191, 67, 71, 11,
            209, 191, 86, 91, 196, 68, 98, 214,
        ];
        let mut sig_as_scalars = [Scalar::zero(); 64];
        for i in 0..64 {
            sig_as_scalars[i] = sig[i].into()
        }
        let message: Vec<Scalar> = vec![
            0.into(),
            1.into(),
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            7.into(),
            8.into(),
            9.into(),
        ];
        let mut witness_values = Vec::new();
        witness_values.extend(message);
        witness_values.push(pub_x);
        witness_values.push(pub_y);
        witness_values.extend(&sig_as_scalars);
        witness_values.push(Scalar::zero());

        let case_1 = WitnessResult {
            witness: Assignments(witness_values),
            public_inputs: None,
            result: true,
        };

        test_circuit(constraint_system, vec![case_1]);
    }
    #[test]
    fn test_ped_constraints() {
        let constraint = PedersenConstraint {
            inputs: vec![1, 2],
            result_x: 3,
            result_y: 4,
        };

        let x_constraint = Constraint {
            a: 3,
            b: 3,
            c: 3,
            qm: Scalar::zero(),
            ql: Scalar::one(),
            qr: Scalar::zero(),
            qo: Scalar::zero(),
            qc: -Scalar::from_hex(
                "0x108800e84e0f1dafb9fdf2e4b5b311fd59b8b08eaf899634c59cc985b490234b",
            )
            .unwrap(),
        };
        let y_constraint = Constraint {
            a: 4,
            b: 4,
            c: 4,
            qm: Scalar::zero(),
            ql: Scalar::one(),
            qr: Scalar::zero(),
            qo: Scalar::zero(),
            qc: -Scalar::from_hex(
                "0x2d43ef68df82e0adf74fed92b1bc950670b9806afcfbcda08bb5baa6497bdf14",
            )
            .unwrap(),
        };

        let constraint_system = ConstraintSystem {
            var_num: 100,
            public_inputs: vec![],
            logic_constraints: vec![],
            range_constraints: vec![],
            sha256_constraints: vec![],
            merkle_membership_constraints: vec![],
            schnorr_constraints: vec![],
            blake2s_constraints: vec![],
            pedersen_constraints: vec![constraint],
            hash_to_field_constraints: vec![],
            constraints: vec![x_constraint, y_constraint],
            ecdsa_secp256k1_constraints: vec![],
            fixed_base_scalar_mul_constraints: vec![],
            insert_merkle_constraints: vec![],
        };

        let scalar_0 = Scalar::from_hex("0x00").unwrap();
        let scalar_1 = Scalar::from_hex("0x01").unwrap();

        let mut witness_values = Vec::new();
        witness_values.push(scalar_0);
        witness_values.push(scalar_1);
        // witness_values.push(Scalar::zero());

        let case_1 = WitnessResult {
            witness: Assignments(witness_values),
            public_inputs: None,
            result: true,
        };

        test_circuit(constraint_system, vec![case_1]);
    }

    #[derive(Clone, Debug)]
    struct WitnessResult {
        witness: WitnessAssignments,
        public_inputs: Option<Assignments>,
        result: bool,
    }

    fn test_circuit(constraint_system: ConstraintSystem, test_cases: Vec<WitnessResult>) {
        let mut sc = StandardComposer::new(constraint_system);
        for test_case in test_cases.into_iter() {
            let proof = sc.create_proof(test_case.witness);
            let verified = sc.verify(&proof, test_case.public_inputs);
            assert_eq!(verified, test_case.result);
        }
    }
}
