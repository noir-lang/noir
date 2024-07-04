use crate::components::private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer;
use dep::types::{
    abis::{
    accumulated_data::combined_accumulated_data::CombinedAccumulatedData, gas::Gas,
    kernel_circuit_public_inputs::{KernelCircuitPublicInputs, PrivateKernelCircuitPublicInputs},
    log_hash::{ScopedEncryptedLogHash, NoteLogHash, ScopedLogHash}, note_hash::ScopedNoteHash,
    nullifier::ScopedNullifier
},
    constants::{DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, L2_GAS_PER_NOTE_HASH, L2_GAS_PER_NULLIFIER, L2_GAS_PER_LOG_BYTE},
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
        data.unencrypted_logs_hash = compute_tx_logs_hash(source.unencrypted_logs_hashes.storage.map(|l: ScopedLogHash| l.expose_to_public()));
        data.note_encrypted_log_preimages_length = source.note_encrypted_logs_hashes.storage.fold(0, |len, l: NoteLogHash| len + l.length);
        data.encrypted_log_preimages_length = source.encrypted_logs_hashes.storage.fold(0, |len, l: ScopedEncryptedLogHash| len + l.log_hash.length);
        data.unencrypted_log_preimages_length = source.unencrypted_logs_hashes.storage.fold(0, |len, l: ScopedLogHash| len + l.log_hash.length);
        data.gas_used = self.meter_gas_used(data);
        data
    }

    fn meter_gas_used(self, data: CombinedAccumulatedData) -> Gas {
        let mut metered_da_bytes = 0;
        let mut metered_l2_gas = 0;

        let data_builder = self.output_composer.public_inputs.end;
        // IMPORTANT: Must use data_builder.__.len(), which is the the number of items pushed to the BoundedVec. 
        // Do not use data.__.len(), which is the array's max length.
        metered_da_bytes += data_builder.note_hashes.len() * DA_BYTES_PER_FIELD;
        metered_l2_gas += data_builder.note_hashes.len() * L2_GAS_PER_NOTE_HASH;

        metered_da_bytes += data_builder.nullifiers.len() * DA_BYTES_PER_FIELD;
        metered_l2_gas += data_builder.nullifiers.len() * L2_GAS_PER_NULLIFIER;

        metered_da_bytes += data_builder.l2_to_l1_msgs.len() * DA_BYTES_PER_FIELD;

        metered_da_bytes += data.note_encrypted_log_preimages_length as u32;
        metered_l2_gas += data.note_encrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

        metered_da_bytes += data.encrypted_log_preimages_length as u32;
        metered_l2_gas += data.encrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

        metered_da_bytes += data.unencrypted_log_preimages_length as u32;
        metered_l2_gas += data.unencrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

        let teardown_gas = self.output_composer.public_inputs.constants.tx_context.gas_settings.teardown_gas_limits;
        Gas::new(metered_da_bytes * DA_GAS_PER_BYTE, metered_l2_gas)
            + Gas::tx_overhead()
            + teardown_gas
    }
}
