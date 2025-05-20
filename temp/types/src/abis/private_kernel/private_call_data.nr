use crate::{
    abis::{
        private_circuit_public_inputs::PrivateCircuitPublicInputs,
        protocol_contract_leaf_preimage::ProtocolContractLeafPreimage,
    },
    address::SaltedInitializationHash,
    constants::{
        FUNCTION_TREE_HEIGHT, PROOF_TYPE_OINK, PROOF_TYPE_PG, PROTOCOL_CONTRACT_TREE_HEIGHT,
        PUBLIC_DATA_TREE_HEIGHT,
    },
    data::public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage,
    merkle_tree::membership::MembershipWitness,
    proof::verification_key::ClientIVCVerificationKey,
    public_keys::PublicKeys,
};

pub struct PrivateCallData {
    pub public_inputs: PrivateCircuitPublicInputs,
    pub vk: ClientIVCVerificationKey,
    pub verification_key_hints: PrivateVerificationKeyHints,
}

impl PrivateCallData {
    pub fn verify(self, is_first_app: bool) {
        let proof_type = if is_first_app {
            PROOF_TYPE_OINK
        } else {
            PROOF_TYPE_PG
        };
        // Client IVC public inputs are linked in the backend via the databus
        std::verify_proof_with_type(self.vk.key, [], [], 0, proof_type);
    }
}

pub struct PrivateVerificationKeyHints {
    pub salted_initialization_hash: SaltedInitializationHash,
    pub public_keys: PublicKeys,
    pub contract_class_artifact_hash: Field,
    pub contract_class_public_bytecode_commitment: Field,
    pub function_leaf_membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,
    pub protocol_contract_membership_witness: MembershipWitness<PROTOCOL_CONTRACT_TREE_HEIGHT>,
    pub protocol_contract_leaf: ProtocolContractLeafPreimage,
    pub updated_class_id_witness: MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>,
    pub updated_class_id_leaf: PublicDataTreeLeafPreimage,
    pub updated_class_id_shared_mutable_values: [Field; 3],
}

pub struct PrivateCallDataWithoutPublicInputs {
    pub vk: ClientIVCVerificationKey,
    pub verification_key_hints: PrivateVerificationKeyHints,
}

impl PrivateCallDataWithoutPublicInputs {
    pub fn to_private_call_data(
        self,
        public_inputs: PrivateCircuitPublicInputs,
    ) -> PrivateCallData {
        PrivateCallData {
            public_inputs,
            vk: self.vk,
            verification_key_hints: self.verification_key_hints,
        }
    }
}
