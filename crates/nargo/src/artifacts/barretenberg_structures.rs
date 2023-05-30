use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, FunctionInput, MemoryBlock};
use acvm::acir::circuit::{Circuit, Opcode};
use acvm::acir::native_types::Expression;
use acvm::acir::BlackBoxFunc;
use acvm::FieldElement;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use thiserror::Error;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Error)]
pub(crate) enum Error {
    // NOTE:: only one being used at the moment
    #[error("Malformed Black Box Function: {0} - {1}")]
    MalformedBlackBoxFunc(BlackBoxFunc, String),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct Assignments(Vec<FieldElement>);

// This is a separate impl so the constructor can get the wasm_bindgen macro in the future
impl Assignments {
    #[allow(dead_code)]
    pub(crate) fn new() -> Assignments {
        Assignments::default()
    }

    // pub(crate) fn len(&self) -> usize {
    //     self.0.len()
    // }
}

// impl Assignments {
//     pub(crate) fn to_bytes(&self) -> Vec<u8> {
//         let mut buffer = Vec::new();

//         let witness_len = self.0.len() as u32;
//         buffer.extend_from_slice(&witness_len.to_be_bytes());

//         for assignment in self.0.iter() {
//             buffer.extend_from_slice(&assignment.to_be_bytes());
//         }

//         buffer
//     }

//     pub(crate) fn is_empty(&self) -> bool {
//         self.0.is_empty()
//     }
// }

impl IntoIterator for Assignments {
    type Item = FieldElement;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<FieldElement>> for Assignments {
    fn from(w: Vec<FieldElement>) -> Assignments {
        Assignments(w)
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct Constraint {
    pub(crate) a: i32,
    pub(crate) b: i32,
    pub(crate) c: i32,
    pub(crate) qm: FieldElement,
    pub(crate) ql: FieldElement,
    pub(crate) qr: FieldElement,
    pub(crate) qo: FieldElement,
    pub(crate) qc: FieldElement,
}

impl Default for Constraint {
    fn default() -> Self {
        Constraint {
            a: 0,
            b: 0,
            c: 0,
            qm: FieldElement::zero(),
            ql: FieldElement::zero(),
            qr: FieldElement::zero(),
            qo: FieldElement::zero(),
            qc: FieldElement::zero(),
        }
    }
}

impl Constraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // serialize Wires
        buffer.extend_from_slice(&self.a.to_be_bytes());
        buffer.extend_from_slice(&self.b.to_be_bytes());
        buffer.extend_from_slice(&self.c.to_be_bytes());

        // serialize selectors
        buffer.extend_from_slice(&self.qm.to_be_bytes());
        buffer.extend_from_slice(&self.ql.to_be_bytes());
        buffer.extend_from_slice(&self.qr.to_be_bytes());
        buffer.extend_from_slice(&self.qo.to_be_bytes());
        buffer.extend_from_slice(&self.qc.to_be_bytes());

        buffer
    }

    fn set_linear_term(&mut self, x: FieldElement, witness: i32) {
        if self.a == 0 || self.a == witness {
            self.a = witness;
            self.ql = x;
        } else if self.b == 0 || self.b == witness {
            self.b = witness;
            self.qr = x;
        } else if self.c == 0 || self.c == witness {
            self.c = witness;
            self.qo = x;
        } else {
            unreachable!("Cannot assign linear term to a constrain of width 3");
        }
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RangeConstraint {
    pub(crate) a: i32,
    pub(crate) num_bits: i32,
}

impl RangeConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serializing Wires
        buffer.extend_from_slice(&self.a.to_be_bytes());
        buffer.extend_from_slice(&self.num_bits.to_be_bytes());

        buffer
    }
}
#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct EcdsaConstraint {
    pub(crate) hashed_message: Vec<i32>,
    // Required until Serde adopts const generics: https://github.com/serde-rs/serde/issues/1937
    #[serde(with = "BigArray")]
    pub(crate) signature: [i32; 64],
    pub(crate) public_key_x: [i32; 32],
    pub(crate) public_key_y: [i32; 32],
    pub(crate) result: i32,
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
#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct SchnorrConstraint {
    pub(crate) message: Vec<i32>,
    // Required until Serde adopts const generics: https://github.com/serde-rs/serde/issues/1937
    #[serde(with = "BigArray")]
    pub(crate) signature: [i32; 64],
    pub(crate) public_key_x: i32,
    pub(crate) public_key_y: i32,
    pub(crate) result: i32,
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
#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct ComputeMerkleRootConstraint {
    pub(crate) hash_path: Vec<i32>,
    pub(crate) leaf: i32,
    pub(crate) index: i32,
    pub(crate) result: i32,
}

impl ComputeMerkleRootConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let hash_path_len = self.hash_path.len() as u32;

        buffer.extend_from_slice(&hash_path_len.to_be_bytes());
        for constraint in self.hash_path.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        buffer.extend_from_slice(&self.leaf.to_be_bytes());
        buffer.extend_from_slice(&self.result.to_be_bytes());
        buffer.extend_from_slice(&self.index.to_be_bytes());

        buffer
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct Sha256Constraint {
    pub(crate) inputs: Vec<(i32, i32)>,
    pub(crate) result: [i32; 32],
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
#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct Blake2sConstraint {
    pub(crate) inputs: Vec<(i32, i32)>,
    pub(crate) result: [i32; 32],
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
#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct HashToFieldConstraint {
    pub(crate) inputs: Vec<(i32, i32)>,
    pub(crate) result: i32,
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

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct Keccak256Constraint {
    pub(crate) inputs: Vec<(i32, i32)>,
    pub(crate) result: [i32; 32],
}

impl Keccak256Constraint {
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

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct Keccak256VarConstraint {
    pub(crate) inputs: Vec<(i32, i32)>,
    pub(crate) result: [i32; 32],
    pub(crate) var_message_size: i32,
}

impl Keccak256VarConstraint {
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

        buffer.extend_from_slice(&self.var_message_size.to_be_bytes());

        buffer
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct PedersenConstraint {
    pub(crate) inputs: Vec<i32>,
    pub(crate) result_x: i32,
    pub(crate) result_y: i32,
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
#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct FixedBaseScalarMulConstraint {
    pub(crate) scalar: i32,
    pub(crate) pubkey_x: i32,
    pub(crate) pubkey_y: i32,
}

impl FixedBaseScalarMulConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serializing Wires
        buffer.extend_from_slice(&self.scalar.to_be_bytes());
        buffer.extend_from_slice(&self.pubkey_x.to_be_bytes());
        buffer.extend_from_slice(&self.pubkey_y.to_be_bytes());

        buffer
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct LogicConstraint {
    pub(crate) a: i32,
    pub(crate) b: i32,
    pub(crate) result: i32,
    pub(crate) num_bits: i32,
    pub(crate) is_xor_gate: bool,
}

impl LogicConstraint {
    pub(crate) fn and(a: i32, b: i32, result: i32, num_bits: i32) -> LogicConstraint {
        LogicConstraint { a, b, result, num_bits, is_xor_gate: false }
    }
    pub(crate) fn xor(a: i32, b: i32, result: i32, num_bits: i32) -> LogicConstraint {
        LogicConstraint { a, b, result, num_bits, is_xor_gate: true }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // Serializing Wires
        buffer.extend_from_slice(&self.a.to_be_bytes());
        buffer.extend_from_slice(&self.b.to_be_bytes());
        buffer.extend_from_slice(&self.result.to_be_bytes());
        buffer.extend_from_slice(&self.num_bits.to_be_bytes());
        buffer.extend_from_slice(&i32::to_be_bytes(self.is_xor_gate as i32));

        buffer
    }
}

#[derive(Clone, Hash, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ConstraintSystem {
    var_num: u32,
    public_inputs: Vec<u32>,

    logic_constraints: Vec<LogicConstraint>,
    range_constraints: Vec<RangeConstraint>,
    sha256_constraints: Vec<Sha256Constraint>,
    compute_merkle_root_constraints: Vec<ComputeMerkleRootConstraint>,
    schnorr_constraints: Vec<SchnorrConstraint>,
    ecdsa_secp256k1_constraints: Vec<EcdsaConstraint>,
    blake2s_constraints: Vec<Blake2sConstraint>,
    block_constraints: Vec<BlockConstraint>,
    keccak_constraints: Vec<Keccak256Constraint>,
    keccak_var_constraints: Vec<Keccak256VarConstraint>,
    pedersen_constraints: Vec<PedersenConstraint>,
    hash_to_field_constraints: Vec<HashToFieldConstraint>,
    fixed_base_scalar_mul_constraints: Vec<FixedBaseScalarMulConstraint>,
    recursion_constraints: Vec<RecursionConstraint>,
    constraints: Vec<Constraint>,
}

// This is a separate impl so the constructor can get the wasm_bindgen macro in the future
impl ConstraintSystem {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        ConstraintSystem::default()
    }
}

// Builder-style impl, but we use all data types that can be defaulted so we don't need a separate builder struct
#[allow(dead_code)]
#[cfg(test)]
impl ConstraintSystem {
    pub(crate) fn var_num(mut self, var_num: u32) -> Self {
        self.var_num = var_num;
        self
    }

    pub(crate) fn public_inputs(mut self, public_inputs: Vec<u32>) -> Self {
        self.public_inputs = public_inputs;
        self
    }

    pub(crate) fn logic_constraints(mut self, logic_constraints: Vec<LogicConstraint>) -> Self {
        self.logic_constraints = logic_constraints;
        self
    }

    pub(crate) fn range_constraints(mut self, range_constraints: Vec<RangeConstraint>) -> Self {
        self.range_constraints = range_constraints;
        self
    }

    pub(crate) fn sha256_constraints(mut self, sha256_constraints: Vec<Sha256Constraint>) -> Self {
        self.sha256_constraints = sha256_constraints;
        self
    }

    pub(crate) fn compute_merkle_root_constraints(
        mut self,
        compute_merkle_root_constraints: Vec<ComputeMerkleRootConstraint>,
    ) -> Self {
        self.compute_merkle_root_constraints = compute_merkle_root_constraints;
        self
    }

    pub(crate) fn schnorr_constraints(
        mut self,
        schnorr_constraints: Vec<SchnorrConstraint>,
    ) -> Self {
        self.schnorr_constraints = schnorr_constraints;
        self
    }

    pub(crate) fn ecdsa_secp256k1_constraints(
        mut self,
        ecdsa_secp256k1_constraints: Vec<EcdsaConstraint>,
    ) -> Self {
        self.ecdsa_secp256k1_constraints = ecdsa_secp256k1_constraints;
        self
    }

    pub(crate) fn blake2s_constraints(
        mut self,
        blake2s_constraints: Vec<Blake2sConstraint>,
    ) -> Self {
        self.blake2s_constraints = blake2s_constraints;
        self
    }

    pub(crate) fn keccak256_constraints(
        mut self,
        keccak256_constraints: Vec<Keccak256Constraint>,
    ) -> Self {
        self.keccak_constraints = keccak256_constraints;
        self
    }

    pub(crate) fn pedersen_constraints(
        mut self,
        pedersen_constraints: Vec<PedersenConstraint>,
    ) -> Self {
        self.pedersen_constraints = pedersen_constraints;
        self
    }

    pub(crate) fn hash_to_field_constraints(
        mut self,
        hash_to_field_constraints: Vec<HashToFieldConstraint>,
    ) -> Self {
        self.hash_to_field_constraints = hash_to_field_constraints;
        self
    }

    pub(crate) fn fixed_base_scalar_mul_constraints(
        mut self,
        fixed_base_scalar_mul_constraints: Vec<FixedBaseScalarMulConstraint>,
    ) -> Self {
        self.fixed_base_scalar_mul_constraints = fixed_base_scalar_mul_constraints;
        self
    }

    pub(crate) fn recursion_constraints(
        mut self,
        recursion_constraints: Vec<RecursionConstraint>,
    ) -> Self {
        self.recursion_constraints = recursion_constraints;
        self
    }

    pub(crate) fn constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    pub(crate) fn block_constraints(mut self, block_constraints: Vec<BlockConstraint>) -> Self {
        self.block_constraints = block_constraints;
        self
    }
}

impl ConstraintSystem {
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();

        // Push lengths onto the buffer
        buffer.extend_from_slice(&self.var_num.to_be_bytes());

        let pi_len = self.public_inputs.len() as u32;
        buffer.extend_from_slice(&pi_len.to_be_bytes());
        for pub_input in self.public_inputs.iter() {
            buffer.extend_from_slice(&pub_input.to_be_bytes());
        }

        // Serialize each Logic constraint
        let logic_constraints_len = self.logic_constraints.len() as u32;
        buffer.extend_from_slice(&logic_constraints_len.to_be_bytes());
        for constraint in self.logic_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Range constraint
        let range_constraints_len = self.range_constraints.len() as u32;
        buffer.extend_from_slice(&range_constraints_len.to_be_bytes());
        for constraint in self.range_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Sha256 constraint
        let sha256_constraints_len = self.sha256_constraints.len() as u32;
        buffer.extend_from_slice(&sha256_constraints_len.to_be_bytes());
        for constraint in self.sha256_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Compute Merkle Root constraint
        let compute_merkle_root_constraints_len = self.compute_merkle_root_constraints.len() as u32;
        buffer.extend_from_slice(&compute_merkle_root_constraints_len.to_be_bytes());
        for constraint in self.compute_merkle_root_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Schnorr constraint
        let schnorr_len = self.schnorr_constraints.len() as u32;
        buffer.extend_from_slice(&schnorr_len.to_be_bytes());
        for constraint in self.schnorr_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each ECDSA constraint
        let ecdsa_len = self.ecdsa_secp256k1_constraints.len() as u32;
        buffer.extend_from_slice(&ecdsa_len.to_be_bytes());
        for constraint in self.ecdsa_secp256k1_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Blake2s constraint
        let blake2s_len = self.blake2s_constraints.len() as u32;
        buffer.extend_from_slice(&blake2s_len.to_be_bytes());
        for constraint in self.blake2s_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Keccak constraint
        let keccak_len = self.keccak_constraints.len() as u32;
        buffer.extend_from_slice(&keccak_len.to_be_bytes());
        for constraint in self.keccak_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Keccak Var constraint
        let keccak_var_len = self.keccak_var_constraints.len() as u32;
        buffer.extend_from_slice(&keccak_var_len.to_be_bytes());
        for constraint in self.keccak_var_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Pedersen constraint
        let pedersen_len = self.pedersen_constraints.len() as u32;
        buffer.extend_from_slice(&pedersen_len.to_be_bytes());
        for constraint in self.pedersen_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each HashToField constraint
        let h2f_len = self.hash_to_field_constraints.len() as u32;
        buffer.extend_from_slice(&h2f_len.to_be_bytes());
        for constraint in self.hash_to_field_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each HashToField constraint
        let fixed_base_scalar_mul_len = self.fixed_base_scalar_mul_constraints.len() as u32;
        buffer.extend_from_slice(&fixed_base_scalar_mul_len.to_be_bytes());
        for constraint in self.fixed_base_scalar_mul_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        let recursion_constraints_len = self.recursion_constraints.len() as u32;
        buffer.extend_from_slice(&recursion_constraints_len.to_be_bytes());
        for constraint in self.recursion_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Arithmetic constraint
        let constraints_len = self.constraints.len() as u32;
        buffer.extend_from_slice(&constraints_len.to_be_bytes());
        for constraint in self.constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        // Serialize each Block constraint
        let len = self.block_constraints.len() as u32;
        buffer.extend_from_slice(&len.to_be_bytes());
        for constraint in self.block_constraints.iter() {
            buffer.extend(&constraint.to_bytes());
        }

        buffer
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct MemOpBarretenberg {
    pub(crate) is_store: i8,
    pub(crate) index: Constraint,
    pub(crate) value: Constraint,
}
impl MemOpBarretenberg {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(&self.is_store.to_be_bytes());
        buffer.extend_from_slice(&self.index.to_bytes());
        buffer.extend_from_slice(&self.value.to_bytes());

        buffer
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct BlockConstraint {
    pub(crate) init: Vec<Constraint>,
    pub(crate) trace: Vec<MemOpBarretenberg>,
    pub(crate) is_ram: i8,
}

impl BlockConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let len = self.init.len() as u32;
        buffer.extend_from_slice(&len.to_be_bytes());
        for value in self.init.iter() {
            buffer.extend_from_slice(&value.to_bytes());
        }

        let len = self.trace.len() as u32;
        buffer.extend_from_slice(&len.to_be_bytes());
        for op in self.trace.iter() {
            buffer.extend_from_slice(&op.to_bytes());
        }
        buffer.extend_from_slice(&self.is_ram.to_be_bytes());

        buffer
    }

    fn from_memory_block(b: &MemoryBlock, is_ram_block: bool) -> BlockConstraint {
        let mut init = Vec::new();
        let mut trace = Vec::new();
        let len = b.len as usize;
        for op in b.trace.iter().take(len) {
            assert_eq!(op.operation, Expression::one());
            init.push(serialize_arithmetic_gates(&op.value));
        }
        for op in b.trace.iter().skip(len) {
            let index = serialize_arithmetic_gates(&op.index);
            let value = serialize_arithmetic_gates(&op.value);
            let bb_op = MemOpBarretenberg {
                is_store: op.operation.to_const().unwrap().to_u128() as i8,
                index,
                value,
            };
            trace.push(bb_op);
        }
        let is_ram = i8::from(is_ram_block);
        BlockConstraint { init, trace, is_ram }
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RecursionConstraint {
    pub(crate) key: Vec<i32>,   // UP size is 115
    pub(crate) proof: Vec<i32>, // UP size is 94
    pub(crate) public_inputs: Vec<i32>,
    pub(crate) key_hash: i32,
    pub(crate) input_aggregation_object: [i32; 16],
    pub(crate) output_aggregation_object: [i32; 16],
    pub(crate) nested_aggregation_object: [i32; 16],
}

impl RecursionConstraint {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let vk_len = (self.key.len()) as u32;
        buffer.extend_from_slice(&vk_len.to_be_bytes());
        for constraint in self.key.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        let proof_len = (self.proof.len()) as u32;
        buffer.extend_from_slice(&proof_len.to_be_bytes());
        for constraint in self.proof.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        let public_inputs_len = (self.public_inputs.len()) as u32;
        buffer.extend_from_slice(&public_inputs_len.to_be_bytes());
        for constraint in self.public_inputs.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        buffer.extend_from_slice(&self.key_hash.to_be_bytes());

        // The aggregation objects are both array's in barretenberg
        // Thus, we do not need to write the length
        for constraint in self.input_aggregation_object.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        for constraint in self.output_aggregation_object.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        for constraint in self.nested_aggregation_object.iter() {
            buffer.extend_from_slice(&constraint.to_be_bytes());
        }

        buffer
    }
}

impl TryFrom<&Circuit> for ConstraintSystem {
    type Error = Error;
    /// Converts an `IR` into the `StandardFormat` constraint system
    fn try_from(circuit: &Circuit) -> Result<Self, Self::Error> {
        // Create constraint system
        let mut constraints: Vec<Constraint> = Vec::new();
        let mut range_constraints: Vec<RangeConstraint> = Vec::new();
        let mut logic_constraints: Vec<LogicConstraint> = Vec::new();
        let mut sha256_constraints: Vec<Sha256Constraint> = Vec::new();
        let mut blake2s_constraints: Vec<Blake2sConstraint> = Vec::new();
        let mut block_constraints: Vec<BlockConstraint> = Vec::new();
        let mut keccak_constraints: Vec<Keccak256Constraint> = Vec::new();
        let keccak_var_constraints: Vec<Keccak256VarConstraint> = Vec::new();
        let mut pedersen_constraints: Vec<PedersenConstraint> = Vec::new();
        // ACVM doesn't generate `ComputeMerkleRootConstraint`s anymore.
        // We maintain this to maintain the serialization format.
        let compute_merkle_root_constraints: Vec<ComputeMerkleRootConstraint> = Vec::new();
        let mut schnorr_constraints: Vec<SchnorrConstraint> = Vec::new();
        let mut ecdsa_secp256k1_constraints: Vec<EcdsaConstraint> = Vec::new();
        let mut fixed_base_scalar_mul_constraints: Vec<FixedBaseScalarMulConstraint> = Vec::new();
        let mut hash_to_field_constraints: Vec<HashToFieldConstraint> = Vec::new();
        let mut recursion_constraints: Vec<RecursionConstraint> = Vec::new();

        for gate in circuit.opcodes.iter() {
            match gate {
                Opcode::Arithmetic(expression) => {
                    let constraint = serialize_arithmetic_gates(expression);
                    constraints.push(constraint);
                }
                Opcode::BlackBoxFuncCall(gadget_call) => {
                    match gadget_call {
                        BlackBoxFuncCall::RANGE { input } => {
                            let witness = input.witness;
                            let num_bits = input.num_bits;

                            let range_constraint = RangeConstraint {
                                a: witness.witness_index() as i32,
                                num_bits: num_bits as i32,
                            };
                            range_constraints.push(range_constraint);
                        }
                        BlackBoxFuncCall::AND { lhs, rhs, output }
                        | BlackBoxFuncCall::XOR { lhs, rhs, output } => {
                            let witness_lhs = lhs.witness;
                            let witness_rhs = rhs.witness;

                            assert_eq!(lhs.num_bits, rhs.num_bits);
                            let num_bits = rhs.num_bits;

                            match gadget_call {
                                BlackBoxFuncCall::AND { .. } => {
                                    let and = LogicConstraint::and(
                                        witness_lhs.witness_index() as i32,
                                        witness_rhs.witness_index() as i32,
                                        output.witness_index() as i32,
                                        num_bits as i32,
                                    );
                                    logic_constraints.push(and);
                                }
                                BlackBoxFuncCall::XOR { .. } => {
                                    let xor = LogicConstraint::xor(
                                        witness_lhs.witness_index() as i32,
                                        witness_rhs.witness_index() as i32,
                                        output.witness_index() as i32,
                                        num_bits as i32,
                                    );
                                    logic_constraints.push(xor);
                                }
                                _ => unreachable!("expected either an AND or XOR opcode"),
                            }
                        }
                        BlackBoxFuncCall::SHA256 { inputs, outputs } => {
                            let mut sha256_inputs: Vec<(i32, i32)> = Vec::new();
                            for input in inputs.iter() {
                                let witness_index = input.witness.witness_index() as i32;
                                let num_bits = input.num_bits as i32;
                                sha256_inputs.push((witness_index, num_bits));
                            }

                            assert_eq!(outputs.len(), 32);

                            let mut outputs_iter = outputs.iter();
                            let mut result = [0i32; 32];
                            for (i, res) in result.iter_mut().enumerate() {
                                let out_byte = outputs_iter.next().ok_or_else(|| {
                                    Error::MalformedBlackBoxFunc(
                                        BlackBoxFunc::SHA256,
                                        format!("Missing rest of output. Tried to get byte {i} but failed"),
                                    )
                                })?;

                                let out_byte_index = out_byte.witness_index() as i32;
                                *res = out_byte_index;
                            }
                            let sha256_constraint =
                                Sha256Constraint { inputs: sha256_inputs, result };

                            sha256_constraints.push(sha256_constraint);
                        }
                        BlackBoxFuncCall::Blake2s { inputs, outputs } => {
                            let mut blake2s_inputs: Vec<(i32, i32)> = Vec::new();
                            for input in inputs.iter() {
                                let witness_index = input.witness.witness_index() as i32;
                                let num_bits = input.num_bits as i32;
                                blake2s_inputs.push((witness_index, num_bits));
                            }

                            assert_eq!(outputs.len(), 32);

                            let mut outputs_iter = outputs.iter();
                            let mut result = [0i32; 32];
                            for (i, res) in result.iter_mut().enumerate() {
                                let out_byte =
                                    outputs_iter.next().ok_or_else(|| {
                                        Error::MalformedBlackBoxFunc(
                                            BlackBoxFunc::Blake2s,
                                            format!("Missing rest of output. Tried to get byte {i} but failed"),
                                        )
                                    })?;

                                let out_byte_index = out_byte.witness_index() as i32;
                                *res = out_byte_index;
                            }
                            let blake2s_constraint =
                                Blake2sConstraint { inputs: blake2s_inputs, result };

                            blake2s_constraints.push(blake2s_constraint);
                        }
                        BlackBoxFuncCall::SchnorrVerify {
                            public_key_x,
                            public_key_y,
                            signature,
                            message: message_inputs,
                            output,
                        } => {
                            // pub_key_x
                            let public_key_x = public_key_x.witness.witness_index() as i32;
                            // pub_key_y
                            let public_key_y = public_key_y.witness.witness_index() as i32;
                            // signature
                            let mut signature_iter = signature.iter();
                            let mut signature = [0i32; 64];
                            for (i, sig) in signature.iter_mut().enumerate() {
                                let sig_byte =
                                    signature_iter.next().ok_or_else(||Error::MalformedBlackBoxFunc(
                                        BlackBoxFunc::SchnorrVerify,
                                        format!("Missing rest of signature. Tried to get byte {i} but failed"),
                                    ))?;
                                let sig_byte_index = sig_byte.witness.witness_index() as i32;
                                *sig = sig_byte_index;
                            }

                            // The rest of the input is the message
                            let mut message = Vec::new();
                            for msg in message_inputs.iter() {
                                let msg_byte_index = msg.witness.witness_index() as i32;
                                message.push(msg_byte_index);
                            }

                            // result
                            let result = output.witness_index() as i32;

                            let constraint = SchnorrConstraint {
                                message,
                                signature,
                                public_key_x,
                                public_key_y,
                                result,
                            };

                            schnorr_constraints.push(constraint);
                        }
                        BlackBoxFuncCall::Pedersen {
                            inputs: gadget_call_inputs,
                            domain_separator: _,
                            outputs,
                        } => {
                            let mut inputs = Vec::new();
                            for scalar in gadget_call_inputs.iter() {
                                let scalar_index = scalar.witness.witness_index() as i32;
                                inputs.push(scalar_index);
                            }

                            assert_eq!(outputs.len(), 2);
                            let result_x = outputs[0].witness_index() as i32;
                            let result_y = outputs[1].witness_index() as i32;

                            let constraint = PedersenConstraint { inputs, result_x, result_y };

                            pedersen_constraints.push(constraint);
                        }
                        BlackBoxFuncCall::HashToField128Security { inputs, output } => {
                            let mut hash_to_field_inputs: Vec<(i32, i32)> = Vec::new();
                            for input in inputs.iter() {
                                let witness_index = input.witness.witness_index() as i32;
                                let num_bits = input.num_bits as i32;
                                hash_to_field_inputs.push((witness_index, num_bits));
                            }

                            let result = output.witness_index() as i32;

                            let hash_to_field_constraint =
                                HashToFieldConstraint { inputs: hash_to_field_inputs, result };

                            hash_to_field_constraints.push(hash_to_field_constraint);
                        }
                        BlackBoxFuncCall::EcdsaSecp256k1 {
                            public_key_x: public_key_x_inputs,
                            public_key_y: public_key_y_inputs,
                            signature: signature_inputs,
                            hashed_message: hashed_message_inputs,
                            output,
                        } => {
                            // public key x
                            let mut public_key_x_inputs = public_key_x_inputs.iter();
                            let mut public_key_x = [0i32; 32];
                            for (i, pkx) in public_key_x.iter_mut().enumerate() {
                                let x_byte = public_key_x_inputs
                                    .next()
                                    .ok_or_else(|| Error::MalformedBlackBoxFunc(
                                        BlackBoxFunc::EcdsaSecp256k1,
                                        format!("Missing rest of `x` component for public key. Tried to get byte {i} but failed"),
                                    ))?;
                                let x_byte_index = x_byte.witness.witness_index() as i32;
                                *pkx = x_byte_index;
                            }

                            // public key y
                            let mut public_key_y_inputs = public_key_y_inputs.iter();
                            let mut public_key_y = [0i32; 32];
                            for (i, pky) in public_key_y.iter_mut().enumerate() {
                                let y_byte = public_key_y_inputs
                                    .next()
                                    .ok_or_else(|| Error::MalformedBlackBoxFunc(
                                        BlackBoxFunc::EcdsaSecp256k1,
                                        format!("Missing rest of `y` component for public key. Tried to get byte {i} but failed"),
                                    ))?;
                                let y_byte_index = y_byte.witness.witness_index() as i32;
                                *pky = y_byte_index;
                            }

                            // signature
                            let mut signature_inputs = signature_inputs.iter();
                            let mut signature = [0i32; 64];
                            for (i, sig) in signature.iter_mut().enumerate() {
                                let sig_byte =
                                    signature_inputs.next().ok_or_else(|| Error::MalformedBlackBoxFunc(
                                        BlackBoxFunc::EcdsaSecp256k1,
                                        format!("Missing rest of signature. Tried to get byte {i} but failed"),
                                    ))?;
                                let sig_byte_index = sig_byte.witness.witness_index() as i32;
                                *sig = sig_byte_index;
                            }

                            // The rest of the input is the message
                            let mut hashed_message = Vec::new();
                            for msg in hashed_message_inputs.iter() {
                                let msg_byte_index = msg.witness.witness_index() as i32;
                                hashed_message.push(msg_byte_index);
                            }

                            // result
                            let result = output.witness_index() as i32;

                            let constraint = EcdsaConstraint {
                                hashed_message,
                                signature,
                                public_key_x,
                                public_key_y,
                                result,
                            };

                            ecdsa_secp256k1_constraints.push(constraint);
                        }
                        BlackBoxFuncCall::FixedBaseScalarMul { input, outputs } => {
                            let scalar = input.witness.witness_index() as i32;

                            assert_eq!(outputs.len(), 2);
                            let pubkey_x = outputs[0].witness_index() as i32;
                            let pubkey_y = outputs[1].witness_index() as i32;

                            let fixed_base_scalar_mul =
                                FixedBaseScalarMulConstraint { scalar, pubkey_x, pubkey_y };

                            fixed_base_scalar_mul_constraints.push(fixed_base_scalar_mul);
                        }
                        BlackBoxFuncCall::Keccak256 { inputs, outputs } => {
                            let mut keccak_inputs: Vec<(i32, i32)> = Vec::new();
                            for input in inputs.iter() {
                                let witness_index = input.witness.witness_index() as i32;
                                let num_bits = input.num_bits as i32;
                                keccak_inputs.push((witness_index, num_bits));
                            }

                            assert_eq!(outputs.len(), 32);

                            let mut outputs_iter = outputs.iter();
                            let mut result = [0i32; 32];
                            for (i, res) in result.iter_mut().enumerate() {
                                let out_byte =
                                    outputs_iter.next().ok_or_else(|| {
                                        Error::MalformedBlackBoxFunc(
                                            BlackBoxFunc::Keccak256,
                                            format!("Missing rest of output. Tried to get byte {i} but failed"),
                                        )
                                    })?;

                                let out_byte_index = out_byte.witness_index() as i32;
                                *res = out_byte_index;
                            }
                            let keccak_constraint =
                                Keccak256Constraint { inputs: keccak_inputs, result };

                            keccak_constraints.push(keccak_constraint);
                        }
                        BlackBoxFuncCall::Keccak256VariableLength { .. } => {
                            // Not handled right now by recursion backend, up-to-date once new cbinds w/ recursion branch is up-to-date with master
                        }
                        BlackBoxFuncCall::RecursiveAggregation {
                            verification_key: key_inputs,
                            proof: proof_inputs,
                            public_inputs: public_inputs_inputs,
                            key_hash,
                            input_aggregation_object,
                            output_aggregation_object,
                        } => {
                            let mut key_inputs = key_inputs.iter();
                            let mut key_array = [0i32; 114];
                            for (i, vk_witness) in key_array.iter_mut().enumerate() {
                                let vk_field = key_inputs.next().unwrap_or_else(|| {
                                    panic!(
                                        "missing rest of vkey. Tried to get field {i} but failed"
                                    )
                                });
                                let vk_field_index = vk_field.witness.witness_index() as i32;
                                *vk_witness = vk_field_index;
                            }
                            let key = key_array.to_vec();

                            let mut proof = Vec::new();
                            for proof_field in proof_inputs.iter() {
                                let proof_field_index = proof_field.witness.witness_index() as i32;
                                proof.push(proof_field_index);
                            }

                            let mut public_inputs = Vec::new();
                            for public_input in public_inputs_inputs.iter() {
                                let public_input_field_index =
                                    public_input.witness.witness_index() as i32;
                                public_inputs.push(public_input_field_index);
                            }

                            // key_hash
                            let key_hash = key_hash.witness.witness_index() as i32;

                            let input_agg_obj_inputs =
                                if let Some(input_aggregation_object) = input_aggregation_object {
                                    input_aggregation_object.clone()
                                } else {
                                    vec![FunctionInput::dummy(); output_aggregation_object.len()]
                                };

                            // input_aggregation_object
                            let mut input_agg_obj_inputs = input_agg_obj_inputs.iter();
                            let mut input_aggregation_object = [0i32; 16];
                            for (i, var) in input_aggregation_object.iter_mut().enumerate() {
                                let var_field = input_agg_obj_inputs.next().unwrap_or_else(|| panic!("missing rest of output aggregation object. Tried to get byte {i} but failed"));
                                let var_field_index = var_field.witness.witness_index() as i32;
                                *var = var_field_index;
                            }

                            // TODO: remove unwrap();
                            let mut nested_aggregation_object: [i32; 16] = [0; 16];
                            if key[5] == 1 {
                                nested_aggregation_object = key[6..22].try_into().unwrap();
                            }

                            // output_aggregation_object
                            let mut outputs_iter = output_aggregation_object.iter();
                            let mut output_aggregation_object = [0i32; 16];
                            for (i, var) in output_aggregation_object.iter_mut().enumerate() {
                                let var_field = outputs_iter.next().unwrap_or_else(|| panic!("missing rest of output aggregation object. Tried to get byte {i} but failed"));
                                let var_field_index = var_field.witness_index() as i32;
                                *var = var_field_index;
                            }

                            let recursion_constraint = RecursionConstraint {
                                key,
                                proof,
                                public_inputs,
                                key_hash,
                                input_aggregation_object,
                                output_aggregation_object,
                                nested_aggregation_object,
                            };
                            recursion_constraints.push(recursion_constraint);
                        }
                    };
                }
                Opcode::Directive(_) | Opcode::Oracle(_) => {
                    // Directives & Oracles are only needed by the pwg
                }
                Opcode::Block(_) | Opcode::Brillig(_) => {
                    // Block is managed by ACVM
                }
                Opcode::RAM(block) => {
                    block_constraints.push(BlockConstraint::from_memory_block(block, true));
                }
                Opcode::ROM(block) => {
                    block_constraints.push(BlockConstraint::from_memory_block(block, false));
                }
            }
        }

        // Create constraint system
        Ok(ConstraintSystem {
            var_num: circuit.current_witness_index + 1, // number of witnesses is the witness index + 1;
            public_inputs: circuit.public_inputs().indices(),
            logic_constraints,
            range_constraints,
            sha256_constraints,
            compute_merkle_root_constraints,
            pedersen_constraints,
            schnorr_constraints,
            ecdsa_secp256k1_constraints,
            blake2s_constraints,
            block_constraints,
            keccak_constraints,
            keccak_var_constraints,
            hash_to_field_constraints,
            constraints,
            fixed_base_scalar_mul_constraints,
            recursion_constraints,
        })
    }
}

#[allow(non_snake_case)]
fn serialize_arithmetic_gates(gate: &Expression) -> Constraint {
    let mut cs = Constraint::default();
    // check mul gate
    if !gate.mul_terms.is_empty() {
        let mul_term = &gate.mul_terms[0];
        cs.qm = mul_term.0;

        // Get wL term
        let wL = &mul_term.1;
        cs.a = wL.witness_index() as i32;

        // Get wR term
        let wR = &mul_term.2;
        cs.b = wR.witness_index() as i32;
    }

    for term in &gate.linear_combinations {
        cs.set_linear_term(term.0, term.1.witness_index() as i32);
    }

    // Add the qc term
    cs.qc = gate.q_c;
    cs
}

#[cfg(test)]
mod tests {
    use crate::artifacts::barretenberg_structures::serialize_arithmetic_gates;
    use acvm::acir::native_types::{Expression, Witness};
    use acvm::FieldElement;

    #[test]
    fn serialize_expression() {
        let x1 = Witness::new(1);
        let x3 = Witness::new(3);
        let two = FieldElement::one() + FieldElement::one();
        let e = Expression {
            mul_terms: vec![(FieldElement::one(), x1, x1)],
            linear_combinations: vec![(two, x1), (-FieldElement::one(), x3)],
            q_c: FieldElement::one(),
        };
        let constrain = serialize_arithmetic_gates(&e);
        assert_eq!(constrain.a, 1);
        assert_eq!(constrain.b, 1);
        assert_eq!(constrain.c, 3);

        let x2 = Witness::new(2);
        let x8 = Witness::new(8);
        let e = Expression {
            mul_terms: vec![(-FieldElement::one(), x1, x2)],
            linear_combinations: vec![(-FieldElement::one(), x8)],
            q_c: FieldElement::zero(),
        };
        let constrain = serialize_arithmetic_gates(&e);
        assert_eq!(constrain.a, 1);
        assert_eq!(constrain.b, 2);
        assert_eq!(constrain.c, 8);

        let e = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one(), x8)],
            q_c: FieldElement::zero(),
        };
        let constrain = serialize_arithmetic_gates(&e);
        assert_eq!(constrain.a, 8);
        assert_eq!(constrain.b, 0);
        assert_eq!(constrain.c, 0);

        let e = Expression {
            mul_terms: vec![(FieldElement::one(), x1, x2)],
            linear_combinations: vec![
                (FieldElement::one(), x8),
                (two, x2),
                (-FieldElement::one(), x1),
            ],
            q_c: FieldElement::zero(),
        };
        let constrain = serialize_arithmetic_gates(&e);
        assert_eq!(constrain.a, 1);
        assert_eq!(constrain.b, 2);
        assert_eq!(constrain.c, 8);
    }
}
