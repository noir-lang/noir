use crate::types::{
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

pub(crate) struct PrivateCallData {
    pub(crate) public_inputs: PrivateCircuitPublicInputs,
    pub(crate) vk: ClientIVCVerificationKey,
    pub(crate) verification_key_hints: PrivateVerificationKeyHints,
}

impl PrivateCallData {
    pub(crate) fn verify(self, is_first_app: bool) {
        let proof_type = if is_first_app {
            PROOF_TYPE_OINK
        } else {
            PROOF_TYPE_PG
        };
        // Client IVC public inputs are linked in the backend via the databus
        std::verify_proof_with_type(self.vk.key, [], [], 0, proof_type);
    }
}

pub(crate) struct PrivateVerificationKeyHints {
    pub(crate) salted_initialization_hash: SaltedInitializationHash,
    pub(crate) public_keys: PublicKeys,
    pub(crate) contract_class_artifact_hash: Field,
    pub(crate) contract_class_public_bytecode_commitment: Field,
    pub(crate) function_leaf_membership_witness: MembershipWitness<FUNCTION_TREE_HEIGHT>,
    pub(crate) protocol_contract_membership_witness: MembershipWitness<PROTOCOL_CONTRACT_TREE_HEIGHT>,
    pub(crate) protocol_contract_leaf: ProtocolContractLeafPreimage,
    pub(crate) updated_class_id_witness: MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>,
    pub(crate) updated_class_id_leaf: PublicDataTreeLeafPreimage,
    pub(crate) updated_class_id_shared_mutable_values: [Field; 3],
}

pub(crate) struct PrivateCallDataWithoutPublicInputs {
    pub(crate) vk: ClientIVCVerificationKey,
    pub(crate) verification_key_hints: PrivateVerificationKeyHints,
}

impl PrivateCallDataWithoutPublicInputs {
    pub(crate) fn to_private_call_data(
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
