use crate::types::{
    abis::kernel_circuit_public_inputs::{
        PrivateToPublicKernelCircuitPublicInputs, PrivateToRollupKernelCircuitPublicInputs,
    },
    constants::{PROOF_TYPE_ROLLUP_HONK, ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS},
    proof::{traits::Verifiable, tube_proof::TubeProof, vk_data::VkData},
    traits::Serialize,
};

pub(crate) struct PublicTubeData {
    pub(crate) public_inputs: PrivateToPublicKernelCircuitPublicInputs,
    pub(crate) proof: TubeProof,
    pub(crate) vk_data: VkData<ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS>,
}

impl Verifiable for PublicTubeData {
    fn verify(self) {
        let inputs = PrivateToPublicKernelCircuitPublicInputs::serialize(self.public_inputs);
        std::verify_proof_with_type(
            self.vk_data.vk.key,
            self.proof.fields,
            inputs,
            self.vk_data.vk.hash,
            PROOF_TYPE_ROLLUP_HONK,
        );
    }
}

pub(crate) struct PrivateTubeData {
    pub(crate) public_inputs: PrivateToRollupKernelCircuitPublicInputs,
    pub(crate) proof: TubeProof,
    pub(crate) vk_data: VkData<ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS>,
}

impl Verifiable for PrivateTubeData {
    fn verify(self) {
        let inputs = PrivateToRollupKernelCircuitPublicInputs::serialize(self.public_inputs);
        std::verify_proof_with_type(
            self.vk_data.vk.key,
            self.proof.fields,
            inputs,
            self.vk_data.vk.hash,
            PROOF_TYPE_ROLLUP_HONK,
        );
    }
}
