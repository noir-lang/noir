pub(crate) mod meter_gas_used;

use crate::components::private_kernel_circuit_public_inputs_composer::PrivateKernelCircuitPublicInputsComposer;
use dep::types::{
    abis::{
        accumulated_data::private_to_rollup_accumulated_data::PrivateToRollupAccumulatedData,
        kernel_circuit_public_inputs::{
            PrivateKernelCircuitPublicInputs, PrivateToRollupKernelCircuitPublicInputs,
        },
        log_hash::LogHash,
        note_hash::ScopedNoteHash,
        nullifier::ScopedNullifier,
        private_log::PrivateLogData,
        side_effect::{counted::Counted, scoped::Scoped},
    },
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    traits::Empty,
};
pub use meter_gas_used::meter_gas_used;

pub struct TailOutputComposer {
    output_composer: PrivateKernelCircuitPublicInputsComposer,
}

impl TailOutputComposer {
    pub unconstrained fn new(previous_kernel: PrivateKernelCircuitPublicInputs) -> Self {
        let mut output_composer =
            PrivateKernelCircuitPublicInputsComposer::new_from_previous_kernel(previous_kernel);
        output_composer.sort_ordered_values();

        TailOutputComposer { output_composer }
    }

    pub unconstrained fn finish(self) -> PrivateToRollupKernelCircuitPublicInputs {
        let source = self.output_composer.finish();
        let mut output = PrivateToRollupKernelCircuitPublicInputs::empty();
        output.rollup_validation_requests = source.validation_requests.for_rollup;
        output.end = self.build_private_to_rollup_accumulated_data();
        output.gas_used = meter_gas_used(output.end);
        output.constants = source.constants;
        output.fee_payer = source.fee_payer;
        output
    }

    unconstrained fn build_private_to_rollup_accumulated_data(
        self,
    ) -> PrivateToRollupAccumulatedData {
        let source = self.output_composer.public_inputs.end;
        let mut data = PrivateToRollupAccumulatedData::empty();
        data.note_hashes = source.note_hashes.storage().map(|n: ScopedNoteHash| n.note_hash.value);
        data.nullifiers = source.nullifiers.storage().map(|n: ScopedNullifier| n.nullifier.value);
        data.l2_to_l1_msgs =
            source.l2_to_l1_msgs.storage().map(|m: ScopedL2ToL1Message| m.expose_to_public());
        data.private_logs =
            source.private_logs.storage().map(|l: Scoped<PrivateLogData>| l.expose_to_public());
        data.contract_class_logs_hashes = source
            .contract_class_logs_hashes
            .storage()
            .map(|l: Scoped<Counted<LogHash>>| l.expose_to_public());
        data
    }
}
