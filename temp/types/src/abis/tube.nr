use crate::{
    abis::kernel_circuit_public_inputs::{
        PrivateToPublicKernelCircuitPublicInputs, PrivateToRollupKernelCircuitPublicInputs,
    },
    constants::{PROOF_TYPE_ROLLUP_HONK, ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS},
    proof::{traits::Verifiable, tube_proof::TubeProof, vk_data::VkData},
    traits::Serialize,
};

pub struct PublicTubeData {
    pub public_inputs: PrivateToPublicKernelCircuitPublicInputs,
    pub proof: TubeProof,
    pub vk_data: VkData<ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS>,
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

pub struct PrivateTubeData {
    pub public_inputs: PrivateToRollupKernelCircuitPublicInputs,
    pub proof: TubeProof,
    pub vk_data: VkData<ROLLUP_HONK_VERIFICATION_KEY_LENGTH_IN_FIELDS>,
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
