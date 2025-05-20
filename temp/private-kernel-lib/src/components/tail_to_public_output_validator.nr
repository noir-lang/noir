mod tail_to_public_output_hints;

use crate::components::tail_to_public_output_composer::meter_gas_used;
use dep::types::{
    abis::{
        kernel_circuit_public_inputs::{
            PrivateKernelCircuitPublicInputs, PrivateToPublicKernelCircuitPublicInputs,
        },
        log_hash::LogHash,
        note_hash::ScopedNoteHash,
        nullifier::ScopedNullifier,
        private_log::{PrivateLog, PrivateLogData},
        public_call_request::PublicCallRequest,
        side_effect::{Counted, scoped::Scoped},
    },
    messaging::l2_to_l1_message::ScopedL2ToL1Message,
    utils::arrays::{
        assert_split_sorted_transformed_value_arrays_asc,
        assert_split_sorted_transformed_value_arrays_desc, assert_split_transformed_value_arrays,
    },
};
use tail_to_public_output_hints::{generate_tail_to_public_output_hints, TailToPublicOutputHints};

pub struct TailToPublicOutputValidator {
    output: PrivateToPublicKernelCircuitPublicInputs,
    previous_kernel: PrivateKernelCircuitPublicInputs,
    hints: TailToPublicOutputHints,
}

impl TailToPublicOutputValidator {
    pub fn new(
        output: PrivateToPublicKernelCircuitPublicInputs,
        previous_kernel: PrivateKernelCircuitPublicInputs,
    ) -> Self {
        // Safety: the below hints are constrained by calling .validate(). See private_kernel_tail_to_public for use.
        let hints = unsafe { generate_tail_to_public_output_hints(previous_kernel) };
        TailToPublicOutputValidator { output, previous_kernel, hints }
    }

    pub fn validate(self) {
        self.validate_propagated_values();
        self.validate_propagated_split_values();
        self.validate_propagated_sorted_values();
        self.validate_gas_used();
    }

    fn validate_propagated_values(self) {
        assert_eq(self.output.constants, self.previous_kernel.constants, "mismatch constants");

        assert_eq(
            self.output.rollup_validation_requests,
            self.previous_kernel.validation_requests.for_rollup,
            "mismatch rollup_validation_requests",
        );

        assert_eq(self.output.fee_payer, self.previous_kernel.fee_payer, "mismatch fee_payer");

        assert_eq(
            self.output.public_teardown_call_request,
            self.previous_kernel.public_teardown_call_request,
            "mismatch public_teardown_call_request",
        );
    }

    fn validate_propagated_split_values(self) {
        let split_counter = self.previous_kernel.min_revertible_side_effect_counter;
        let prev_data = self.previous_kernel.end;
        let output_non_revertible = self.output.non_revertible_accumulated_data;
        let output_revertible = self.output.revertible_accumulated_data;

        // note_hashes
        assert_split_transformed_value_arrays(
            prev_data.note_hashes,
            output_non_revertible.note_hashes,
            output_revertible.note_hashes,
            |prev: ScopedNoteHash, out: Field| out == prev.note_hash.value,
            split_counter,
        );

        // nullifiers
        assert_split_transformed_value_arrays(
            prev_data.nullifiers,
            output_non_revertible.nullifiers,
            output_revertible.nullifiers,
            |prev: ScopedNullifier, out: Field| out == prev.nullifier.value,
            split_counter,
        );

        // private_logs
        assert_split_transformed_value_arrays(
            prev_data.private_logs,
            output_non_revertible.private_logs,
            output_revertible.private_logs,
            |prev: Scoped<PrivateLogData>, out: PrivateLog| out == prev.expose_to_public(),
            split_counter,
        );
    }

    fn validate_propagated_sorted_values(self) {
        let split_counter = self.previous_kernel.min_revertible_side_effect_counter;
        let prev_data = self.previous_kernel.end;
        let output_non_revertible = self.output.non_revertible_accumulated_data;
        let output_revertible = self.output.revertible_accumulated_data;
        let hints = self.hints;

        // l2_to_l1_msgs
        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.l2_to_l1_msgs,
            prev_data.l2_to_l1_msgs.map(|log: ScopedL2ToL1Message| log.expose_to_public()),
            split_counter,
            output_non_revertible.l2_to_l1_msgs,
            output_revertible.l2_to_l1_msgs,
            hints.sorted_l2_to_l1_msg_hints,
        );

        // contract_class_logs_hashes
        assert_split_sorted_transformed_value_arrays_asc(
            prev_data.contract_class_logs_hashes,
            prev_data.contract_class_logs_hashes.map(|log: Scoped<Counted<LogHash>>| {
                log.expose_to_public()
            }),
            split_counter,
            output_non_revertible.contract_class_logs_hashes,
            output_revertible.contract_class_logs_hashes,
            hints.sorted_contract_class_log_hash_hints,
        );

        // public_call_requests
        assert_split_sorted_transformed_value_arrays_desc(
            prev_data.public_call_requests,
            prev_data.public_call_requests.map(|cr: Counted<PublicCallRequest>| cr.inner),
            split_counter,
            output_non_revertible.public_call_requests,
            output_revertible.public_call_requests,
            hints.sorted_public_call_request_hints,
        )
    }

    fn validate_gas_used(self) {
        let gas_settings = self.output.constants.tx_context.gas_settings;
        let gas_used = meter_gas_used(
            self.output.non_revertible_accumulated_data,
            self.output.revertible_accumulated_data,
            self.output.public_teardown_call_request,
            gas_settings.teardown_gas_limits,
        );
        assert_eq(self.output.gas_used, gas_used, "incorrect metered gas used");
        assert(gas_used.within(gas_settings.gas_limits), "The gas used exceeds the gas limits");
    }
}
