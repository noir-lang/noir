use crate::components::private_kernel_circuit_public_inputs_composer::create_protocol_nullifier;
use dep::types::{
    abis::{
        kernel_circuit_public_inputs::{
            PrivateKernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsArrayLengths,
        },
        max_block_number::MaxBlockNumber,
        private_circuit_public_inputs::{
            PrivateCircuitPublicInputs, PrivateCircuitPublicInputsArrayLengths,
        },
        private_kernel::private_call_data::PrivateCallData,
    },
    address::AztecAddress,
    constants::DEFAULT_UPDATE_DELAY,
    contract_class_id::ContractClassId,
    shared_mutable::{compute_shared_mutable_block_horizon, SharedMutableValues},
    traits::{is_empty, Packable},
    transaction::tx_request::TxRequest,
    utils::arrays::{
        assert_array_appended, assert_array_appended_and_scoped, assert_array_appended_reversed,
        assert_array_appended_scoped, assert_array_prepended,
    },
};

pub struct PrivateKernelCircuitOutputValidator {
    output: PrivateKernelCircuitPublicInputs,
}

impl PrivateKernelCircuitOutputValidator {
    pub fn new(output: PrivateKernelCircuitPublicInputs) -> Self {
        PrivateKernelCircuitOutputValidator { output }
    }

    pub fn validate_as_first_call(
        self,
        tx_request: TxRequest,
        private_call: PrivateCallData,
        private_call_array_lengths: PrivateCircuitPublicInputsArrayLengths,
        vk_tree_root: Field,
        protocol_contract_tree_root: Field,
        is_private_only: bool,
        first_nullifier_hint: Field,
    ) {
        self.validate_initial_values(
            tx_request,
            private_call,
            vk_tree_root,
            protocol_contract_tree_root,
            is_private_only,
            first_nullifier_hint,
        );
        let mut offsets = PrivateKernelCircuitPublicInputsArrayLengths::empty();
        if first_nullifier_hint == 0 {
            offsets.nullifiers = 1; // The protocol nullifier is not propagated from the private call.
        }
        self.validate_propagated_from_private_call(
            private_call.public_inputs,
            private_call_array_lengths,
            offsets,
            0, // num_popped_call
        );
    }

    pub fn validate_as_inner_call(
        self,
        previous_kernel: PrivateKernelCircuitPublicInputs,
        previous_kernel_array_lengths: PrivateKernelCircuitPublicInputsArrayLengths,
        private_call: PrivateCallData,
        private_call_array_lengths: PrivateCircuitPublicInputsArrayLengths,
    ) {
        self.validate_aggregated_values(previous_kernel, private_call);
        self.validate_propagated_from_previous_kernel(
            previous_kernel,
            previous_kernel_array_lengths,
        );
        self.validate_propagated_from_private_call(
            private_call.public_inputs,
            private_call_array_lengths,
            previous_kernel_array_lengths,
            1, // num_popped_call
        );
    }

    fn validate_initial_values(
        self,
        tx_request: TxRequest,
        private_call: PrivateCallData,
        vk_tree_root: Field,
        protocol_contract_tree_root: Field,
        is_private_only: bool,
        first_nullifier_hint: Field,
    ) {
        // Constants.
        assert_eq(self.output.is_private_only, is_private_only, "mismatch is_private_only");
        assert_eq(self.output.constants.tx_context, tx_request.tx_context, "mismatch tx_context");
        assert_eq(
            self.output.constants.historical_header,
            private_call.public_inputs.historical_header,
            "mismatch historical_header",
        );
        assert_eq(self.output.constants.vk_tree_root, vk_tree_root, "mismatch vk_tree_root");
        assert_eq(
            self.output.constants.protocol_contract_tree_root,
            protocol_contract_tree_root,
            "mismatch protocol_contract_tree_root",
        );

        let protocol_nullifier = create_protocol_nullifier(tx_request);
        // No need to check claimed_first_nullifier if the hint is nonzero, since it's just a prover hint that will be verified in tail.
        // Also, a first_nullifier_hint of 0 with a manipulated claimed_first_nullifier will result in an always failing check in tail.
        if first_nullifier_hint == 0 {
            assert_eq(
                self.output.end.nullifiers[0],
                protocol_nullifier,
                "protocol nullifier must be the tx request nullifier",
            );
        }

        // Others.
        assert_eq(
            self.output.min_revertible_side_effect_counter,
            private_call.public_inputs.min_revertible_side_effect_counter,
            "incorrect initial min_revertible_side_effect_counter",
        );

        let max_block_number = Self::update_max_block_number_for_contract_updates(
            private_call,
            private_call.public_inputs.max_block_number,
        );

        assert_eq(
            self.output.validation_requests.for_rollup.max_block_number,
            max_block_number,
            "incorrect initial max_block_number",
        );
        assert_eq(
            self.output.public_teardown_call_request,
            private_call.public_inputs.public_teardown_call_request,
            "incorrect initial public_teardown_call_request",
        );
        let initial_fee_payer = if private_call.public_inputs.is_fee_payer {
            private_call.public_inputs.call_context.contract_address
        } else {
            AztecAddress::zero()
        };
        assert_eq(self.output.fee_payer, initial_fee_payer, "incorrect initial fee_payer");
    }

    fn validate_aggregated_values(
        self,
        previous_kernel: PrivateKernelCircuitPublicInputs,
        private_call: PrivateCallData,
    ) {
        // min_revertible_side_effect_counter
        let propagated_min_revertible_counter = if previous_kernel
            .min_revertible_side_effect_counter
            != 0 {
            assert(
                private_call.public_inputs.min_revertible_side_effect_counter == 0,
                "cannot overwrite min_revertible_side_effect_counter",
            );
            previous_kernel.min_revertible_side_effect_counter
        } else {
            private_call.public_inputs.min_revertible_side_effect_counter
        };
        assert_eq(
            self.output.min_revertible_side_effect_counter,
            propagated_min_revertible_counter,
            "incorrect output min_revertible_side_effect_counter",
        );

        // max_block_number
        let max_block_number = MaxBlockNumber::min(
            previous_kernel.validation_requests.for_rollup.max_block_number,
            private_call.public_inputs.max_block_number,
        );

        let max_block_number =
            Self::update_max_block_number_for_contract_updates(private_call, max_block_number);

        assert_eq(
            self.output.validation_requests.for_rollup.max_block_number,
            max_block_number,
            "incorrect output max_block_number",
        );

        // public_teardown_call_request
        let propagated_public_teardown_call_request = if !is_empty(
            previous_kernel.public_teardown_call_request,
        ) {
            assert(
                is_empty(private_call.public_inputs.public_teardown_call_request),
                "cannot overwrite public_teardown_call_request",
            );
            previous_kernel.public_teardown_call_request
        } else {
            private_call.public_inputs.public_teardown_call_request
        };
        assert_eq(
            self.output.public_teardown_call_request,
            propagated_public_teardown_call_request,
            "incorrect output public_teardown_call_request",
        );

        // fee_payer
        let propagated_fee_payer = if !is_empty(previous_kernel.fee_payer) {
            assert(!private_call.public_inputs.is_fee_payer, "cannot overwrite fee_payer");
            previous_kernel.fee_payer
        } else if private_call.public_inputs.is_fee_payer {
            private_call.public_inputs.call_context.contract_address
        } else {
            AztecAddress::zero()
        };
        assert_eq(self.output.fee_payer, propagated_fee_payer, "incorrect output fee_payer");
    }

    fn validate_propagated_from_previous_kernel(
        self,
        previous_kernel: PrivateKernelCircuitPublicInputs,
        array_lengths: PrivateKernelCircuitPublicInputsArrayLengths,
    ) {
        assert_eq(
            self.output.is_private_only,
            previous_kernel.is_private_only,
            "mismatch is_private_only",
        );
        assert_eq(
            self.output.claimed_first_nullifier,
            previous_kernel.claimed_first_nullifier,
            "mismatch claimed_first_nullifier",
        );
        assert_eq(self.output.constants, previous_kernel.constants, "mismatch constants");

        assert_eq(
            self.output.validation_requests.split_counter,
            previous_kernel.validation_requests.split_counter,
            "mismatch validation requests split counter",
        );

        assert_array_prepended(
            self.output.validation_requests.note_hash_read_requests,
            previous_kernel.validation_requests.note_hash_read_requests,
            array_lengths.note_hash_read_requests,
        );
        assert_array_prepended(
            self.output.validation_requests.nullifier_read_requests,
            previous_kernel.validation_requests.nullifier_read_requests,
            array_lengths.nullifier_read_requests,
        );
        assert_array_prepended(
            self.output.validation_requests.scoped_key_validation_requests_and_generators,
            previous_kernel.validation_requests.scoped_key_validation_requests_and_generators,
            array_lengths.scoped_key_validation_requests_and_generators,
        );
        assert_array_prepended(
            self.output.end.note_hashes,
            previous_kernel.end.note_hashes,
            array_lengths.note_hashes,
        );
        assert_array_prepended(
            self.output.end.nullifiers,
            previous_kernel.end.nullifiers,
            array_lengths.nullifiers,
        );
        assert_array_prepended(
            self.output.end.l2_to_l1_msgs,
            previous_kernel.end.l2_to_l1_msgs,
            array_lengths.l2_to_l1_msgs,
        );
        assert_array_prepended(
            self.output.end.private_logs,
            previous_kernel.end.private_logs,
            array_lengths.private_logs,
        );
        assert_array_prepended(
            self.output.end.contract_class_logs_hashes,
            previous_kernel.end.contract_class_logs_hashes,
            array_lengths.contract_class_logs_hashes,
        );
        assert_array_prepended(
            self.output.end.public_call_requests,
            previous_kernel.end.public_call_requests,
            array_lengths.public_call_requests,
        );
        // array_lengths.private_call_stack is guaranteed to be greater than 0.
        // It's checked in private_kernel_inner when comparing the top item in the stack with the current private call.
        assert_array_prepended(
            self.output.end.private_call_stack,
            previous_kernel.end.private_call_stack,
            array_lengths.private_call_stack - 1, // Do not copy the top item in the stack.
        );
    }

    fn validate_propagated_from_private_call(
        self,
        private_call: PrivateCircuitPublicInputs,
        array_lengths: PrivateCircuitPublicInputsArrayLengths,
        offsets: PrivateKernelCircuitPublicInputsArrayLengths,
        num_popped_call: u32,
    ) {
        let contract_address = private_call.call_context.contract_address;
        assert_array_appended_scoped(
            self.output.validation_requests.note_hash_read_requests,
            private_call.note_hash_read_requests,
            array_lengths.note_hash_read_requests,
            offsets.note_hash_read_requests,
            contract_address,
        );
        assert_array_appended_scoped(
            self.output.validation_requests.nullifier_read_requests,
            private_call.nullifier_read_requests,
            array_lengths.nullifier_read_requests,
            offsets.nullifier_read_requests,
            contract_address,
        );
        assert_array_appended_scoped(
            self.output.validation_requests.scoped_key_validation_requests_and_generators,
            private_call.key_validation_requests_and_generators,
            array_lengths.key_validation_requests_and_generators,
            offsets.scoped_key_validation_requests_and_generators,
            contract_address,
        );
        assert_array_appended_scoped(
            self.output.end.note_hashes,
            private_call.note_hashes,
            array_lengths.note_hashes,
            offsets.note_hashes,
            contract_address,
        );
        assert_array_appended_scoped(
            self.output.end.nullifiers,
            private_call.nullifiers,
            array_lengths.nullifiers,
            offsets.nullifiers,
            contract_address,
        );
        assert_array_appended_scoped(
            self.output.end.l2_to_l1_msgs,
            private_call.l2_to_l1_msgs,
            array_lengths.l2_to_l1_msgs,
            offsets.l2_to_l1_msgs,
            contract_address,
        );
        assert_array_appended_and_scoped(
            self.output.end.private_logs,
            private_call.private_logs,
            array_lengths.private_logs,
            offsets.private_logs,
            contract_address,
        );
        assert_array_appended_and_scoped(
            self.output.end.contract_class_logs_hashes,
            private_call.contract_class_logs_hashes,
            array_lengths.contract_class_logs_hashes,
            offsets.contract_class_logs_hashes,
            contract_address,
        );
        assert_array_appended(
            self.output.end.public_call_requests,
            private_call.public_call_requests,
            array_lengths.public_call_requests,
            offsets.public_call_requests,
        );
        assert_array_appended_reversed(
            self.output.end.private_call_stack,
            private_call.private_call_requests,
            array_lengths.private_call_requests,
            offsets.private_call_stack - num_popped_call,
        );
    }

    fn update_max_block_number_for_contract_updates(
        private_call: PrivateCallData,
        max_block_number: MaxBlockNumber,
    ) -> MaxBlockNumber {
        if !private_call.public_inputs.call_context.contract_address.is_protocol_contract() {
            let shared_mutable_values: SharedMutableValues<ContractClassId, DEFAULT_UPDATE_DELAY> = Packable::unpack(
                private_call.verification_key_hints.updated_class_id_shared_mutable_values,
            );

            MaxBlockNumber::min(
                max_block_number,
                MaxBlockNumber::new(compute_shared_mutable_block_horizon(
                    shared_mutable_values,
                    private_call.public_inputs.historical_header.global_variables.block_number
                         as u32,
                    )),
                )
            } else {
                max_block_number
            }
        }
}
