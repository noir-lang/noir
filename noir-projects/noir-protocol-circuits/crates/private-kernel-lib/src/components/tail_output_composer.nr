mod meter_gas_used;

use crate::components::{
    private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer,
    tail_output_composer::meter_gas_used::meter_gas_used
};
use dep::types::{
    abis::{
    accumulated_data::combined_accumulated_data::CombinedAccumulatedData,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PrivateKernelCircuitPublicInputs},
    log_hash::{ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash}, note_hash::ScopedNoteHash,
    nullifier::ScopedNullifier
},
    hash::{compute_tx_logs_hash, compute_tx_note_logs_hash},
    messaging::l2_to_l1_message::ScopedL2ToL1Message
};

struct TailOutputComposer {
    output_composer: PrivateKernelCircuitPublicInputsComposer,
}

impl TailOutputComposer {
    pub fn new(previous_kernel: PrivateKernelCircuitPublicInputs) -> Self {
        let mut output_composer = PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(previous_kernel);
        output_composer.sort_and_silo();

        TailOutputComposer { output_composer }
    }

    pub fn finish(self) -> KernelCircuitPublicInputs {
        let source = self.output_composer.finish();
        let mut output = KernelCircuitPublicInputs::empty();
        output.rollup_validation_requests = source.validation_requests.for_rollup;
        output.end = self.build_combined_accumulated_data();
        output.constants = source.constants;
        output.fee_payer = source.fee_payer;
        output
    }

    fn build_combined_accumulated_data(self) -> CombinedAccumulatedData {
        let source = self.output_composer.public_inputs.end;
        let mut data = CombinedAccumulatedData::empty();
        data.note_hashes = source.note_hashes.storage.map(|n: ScopedNoteHash| n.note_hash.value);
        data.nullifiers = source.nullifiers.storage.map(|n: ScopedNullifier| n.nullifier.value);
        data.l2_to_l1_msgs = source.l2_to_l1_msgs.storage.map(|m: ScopedL2ToL1Message| m.message.content);
        data.note_encrypted_logs_hash = compute_tx_note_logs_hash(source.note_encrypted_logs_hashes.storage.map(|l: NoteLogHash| l.expose_to_public()));
        data.encrypted_logs_hash = compute_tx_logs_hash(source.encrypted_logs_hashes.storage.map(|l: ScopedEncryptedLogHash| l.expose_to_public()));
        data.unencrypted_logs_hashes = source.unencrypted_logs_hashes.storage.map(|l: ScopedLogHash| l.expose_to_public());
        data.note_encrypted_log_preimages_length = source.note_encrypted_logs_hashes.storage.fold(0, |len, l: NoteLogHash| len + l.length);
        data.encrypted_log_preimages_length = source.encrypted_logs_hashes.storage.fold(0, |len, l: ScopedEncryptedLogHash| len + l.log_hash.length);
        data.unencrypted_log_preimages_length = source.unencrypted_logs_hashes.storage.fold(0, |len, l: ScopedLogHash| len + l.log_hash.length);
        data.gas_used = meter_gas_used(data, self.output_composer.public_inputs.constants.tx_context.gas_settings);
        data
    }
}
