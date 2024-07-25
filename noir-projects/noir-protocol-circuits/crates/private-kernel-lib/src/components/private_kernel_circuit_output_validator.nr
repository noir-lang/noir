use crate::components::private_kernel_circuit_public_inputs_composer::create_first_nullifier;
use dep::types::{
    abis::{
    kernel_circuit_public_inputs::{PrivateKernelCircuitPublicInputs, PrivateKernelCircuitPublicInputsArrayLengths},
    max_block_number::MaxBlockNumber,
    private_circuit_public_inputs::{PrivateCircuitPublicInputs, PrivateCircuitPublicInputsArrayLengths},
    private_kernel::private_call_data::PrivateCallData, side_effect::Scoped
},
    address::AztecAddress, traits::{Empty, is_empty}, transaction::tx_request::TxRequest
};

fn validate_array_prepended<T, N>(dest: [T; N], source: [T; N], num_source_items: u32) where T: Eq {
    let mut proceed = true;
    for i in 0..source.len() {
        proceed &= i != num_source_items;
        if proceed {
            assert_eq(dest[i], source[i], "source item does not prepend to dest");
        }
    }
}

fn validate_array_appended<T, N, M>(
    dest: [T; N],
    source: [T; M],
    num_source_items: u32,
    num_prepended_items: u32
) where T: Empty + Eq {
    let items_propagated = num_prepended_items + num_source_items;
    assert(items_propagated <= N, "number of total items exceeds limit");
    let mut should_check = false;
    let mut is_non_empty_item = true;
    for i in 0..dest.len() { // Loop through dest instead of source because we also need to check that dest is appended with empty items.
        should_check |= i == num_prepended_items; // Prepended items have been checked in validate_array_prepended() and can be skipped here.
        is_non_empty_item &= i != items_propagated;
        if should_check {
            if is_non_empty_item {
                assert_eq(dest[i], source[i - num_prepended_items], "source item does not append to dest");
            } else {
                assert(is_empty(dest[i]), "output should be appended with empty items");
            }
        }
    }
}

// Similar to validate_array_appended, except that the contract address of the dest items will also be checked.
fn validate_array_appended_scoped<ST, T, N, M>(
    dest: [ST; N],
    source: [T; M],
    num_source_items: u32,
    num_prepended_items: u32,
    contract_address: AztecAddress
) where ST: Scoped<T> + Empty + Eq, T: Eq {
    let items_propagated = num_prepended_items + num_source_items;
    assert(items_propagated <= N, "number of total items exceeds limit");
    let mut should_check = false;
    let mut is_non_empty_item = true;
    for i in 0..dest.len() {
        should_check |= i == num_prepended_items;
        is_non_empty_item &= i != items_propagated;
        if should_check {
            if is_non_empty_item {
                assert_eq(
                    dest[i].inner(), source[i - num_prepended_items], "source item does not append to dest"
                );
                assert_eq(
                    dest[i].contract_address(), contract_address, "propagated contract address does not match"
                );
            } else {
                assert(is_empty(dest[i]), "output should be appended with empty items");
            }
        }
    }
}

// Similar to validate_array_appended, except that the souce items will be appended to dest in reversed order.
fn validate_array_appended_reversed<T, N, M>(
    dest: [T; N],
    source: [T; M],
    num_source_items: u32,
    num_prepended_items: u32
) where T: Empty + Eq {
    let items_propagated = num_prepended_items + num_source_items;
    assert(items_propagated <= N, "number of total items exceeds limit");
    let mut should_check = false;
    let mut is_non_empty_item = true;
    for i in 0..dest.len() {
        should_check |= i == num_prepended_items;
        is_non_empty_item &= i != items_propagated;
        if should_check {
            if is_non_empty_item {
                assert_eq(
                    dest[i], source[items_propagated - i - 1], "source item does not reversed append to dest"
                );
            } else {
                assert(is_empty(dest[i]), "output should be appended with empty items");
            }
        }
    }
}

struct PrivateKernelCircuitOutputValidator {
    output: PrivateKernelCircuitPublicInputs,
}

impl PrivateKernelCircuitOutputValidator {
    pub fn new(output: PrivateKernelCircuitPublicInputs) -> Self {
        PrivateKernelCircuitOutputValidator { output }
    }

    pub fn validate_as_first_call(
        self,
        tx_request: TxRequest,
        private_call: PrivateCircuitPublicInputs,
        private_call_array_lengths: PrivateCircuitPublicInputsArrayLengths,
        vk_tree_root: Field
    ) {
        self.validate_initial_values(tx_request, private_call, vk_tree_root);
        let mut offsets = PrivateKernelCircuitPublicInputsArrayLengths::empty();
        offsets.nullifiers = 1; // The first nullifier is not propagated from the private call.
        self.validate_propagated_from_private_call(
            private_call,
            private_call_array_lengths,
            offsets,
            0 // num_popped_call
        );
    }

    pub fn validate_as_inner_call(
        self,
        previous_kernel: PrivateKernelCircuitPublicInputs,
        previous_kernel_array_lengths: PrivateKernelCircuitPublicInputsArrayLengths,
        private_call: PrivateCircuitPublicInputs,
        private_call_array_lengths: PrivateCircuitPublicInputsArrayLengths
    ) {
        self.validate_aggregated_values(previous_kernel, private_call);
        self.validate_propagated_from_previous_kernel(previous_kernel, previous_kernel_array_lengths);
        self.validate_propagated_from_private_call(
            private_call,
            private_call_array_lengths,
            previous_kernel_array_lengths,
            1 // num_popped_call
        );
    }

    fn validate_initial_values(
        self,
        tx_request: TxRequest,
        private_call: PrivateCircuitPublicInputs,
        vk_tree_root: Field
    ) {
        // Constants.
        assert_eq(self.output.constants.tx_context, tx_request.tx_context, "mismatch tx_context");
        assert_eq(
            self.output.constants.historical_header, private_call.historical_header, "mismatch historical_header"
        );
        assert(is_empty(self.output.constants.global_variables), "constants.global_variables must be empty");
        assert_eq(self.output.constants.vk_tree_root, vk_tree_root);

        // First nullifier.
        let first_nullifier = create_first_nullifier(tx_request);
        assert_eq(
            self.output.end.nullifiers[0], first_nullifier, "first nullifier must be the tx request nullifier"
        );

        // Others.
        assert_eq(
            self.output.min_revertible_side_effect_counter, private_call.min_revertible_side_effect_counter, "incorrect initial min_revertible_side_effect_counter"
        );
        assert_eq(
            self.output.validation_requests.for_rollup.max_block_number, private_call.max_block_number, "incorrect initial max_block_number"
        );
        assert_eq(
            self.output.public_teardown_call_request, private_call.public_teardown_call_request, "incorrect initial public_teardown_call_request"
        );
        let initial_fee_payer = if private_call.is_fee_payer {
            private_call.call_context.storage_contract_address
        } else {
            AztecAddress::zero()
        };
        assert_eq(self.output.fee_payer, initial_fee_payer, "incorrect initial fee_payer");
    }

    fn validate_aggregated_values(
        self,
        previous_kernel: PrivateKernelCircuitPublicInputs,
        private_call: PrivateCircuitPublicInputs
    ) {
        // min_revertible_side_effect_counter
        let propagated_min_revertible_counter = if previous_kernel.min_revertible_side_effect_counter != 0 {
            assert(
                private_call.min_revertible_side_effect_counter == 0, "cannot overwrite min_revertible_side_effect_counter"
            );
            previous_kernel.min_revertible_side_effect_counter
        } else {
            private_call.min_revertible_side_effect_counter
        };
        assert_eq(
            self.output.min_revertible_side_effect_counter, propagated_min_revertible_counter, "incorrect output min_revertible_side_effect_counter"
        );

        // max_block_number
        let max_block_number = MaxBlockNumber::min(
            previous_kernel.validation_requests.for_rollup.max_block_number,
            private_call.max_block_number
        );
        assert_eq(
            self.output.validation_requests.for_rollup.max_block_number, max_block_number, "incorrect output max_block_number"
        );

        // public_teardown_call_request
        let propagated_public_teardown_call_request = if !is_empty(previous_kernel.public_teardown_call_request) {
            assert(
                is_empty(private_call.public_teardown_call_request), "cannot overwrite public_teardown_call_request"
            );
            previous_kernel.public_teardown_call_request
        } else {
            private_call.public_teardown_call_request
        };
        assert_eq(
            self.output.public_teardown_call_request, propagated_public_teardown_call_request, "incorrect output public_teardown_call_request"
        );

        // fee_payer
        let propagated_fee_payer = if !is_empty(previous_kernel.fee_payer) {
            assert(!private_call.is_fee_payer, "cannot overwrite fee_payer");
            previous_kernel.fee_payer
        } else if private_call.is_fee_payer {
            private_call.call_context.storage_contract_address
        } else {
            AztecAddress::zero()
        };
        assert_eq(self.output.fee_payer, propagated_fee_payer, "incorrect output fee_payer");
    }

    fn validate_propagated_from_previous_kernel(
        self,
        previous_kernel: PrivateKernelCircuitPublicInputs,
        array_lengths: PrivateKernelCircuitPublicInputsArrayLengths
    ) {
        assert_eq(self.output.constants, previous_kernel.constants, "mismatch constants");

        validate_array_prepended(
            self.output.validation_requests.note_hash_read_requests,
            previous_kernel.validation_requests.note_hash_read_requests,
            array_lengths.note_hash_read_requests
        );
        validate_array_prepended(
            self.output.validation_requests.nullifier_read_requests,
            previous_kernel.validation_requests.nullifier_read_requests,
            array_lengths.nullifier_read_requests
        );
        validate_array_prepended(
            self.output.validation_requests.scoped_key_validation_requests_and_generators,
            previous_kernel.validation_requests.scoped_key_validation_requests_and_generators,
            array_lengths.scoped_key_validation_requests_and_generators
        );
        validate_array_prepended(
            self.output.end.note_hashes,
            previous_kernel.end.note_hashes,
            array_lengths.note_hashes
        );
        validate_array_prepended(
            self.output.end.nullifiers,
            previous_kernel.end.nullifiers,
            array_lengths.nullifiers
        );
        validate_array_prepended(
            self.output.end.l2_to_l1_msgs,
            previous_kernel.end.l2_to_l1_msgs,
            array_lengths.l2_to_l1_msgs
        );
        validate_array_prepended(
            self.output.end.note_encrypted_logs_hashes,
            previous_kernel.end.note_encrypted_logs_hashes,
            array_lengths.note_encrypted_logs_hashes
        );
        validate_array_prepended(
            self.output.end.encrypted_logs_hashes,
            previous_kernel.end.encrypted_logs_hashes,
            array_lengths.encrypted_logs_hashes
        );
        validate_array_prepended(
            self.output.end.unencrypted_logs_hashes,
            previous_kernel.end.unencrypted_logs_hashes,
            array_lengths.unencrypted_logs_hashes
        );
        validate_array_prepended(
            self.output.end.public_call_requests,
            previous_kernel.end.public_call_requests,
            array_lengths.public_call_requests
        );
        // array_lengths.private_call_stack is guaranteed to be greater than 0.
        // It's checked in private_kernel_inner when comparing the top item in the stack with the current private call.
        validate_array_prepended(
            self.output.end.private_call_stack,
            previous_kernel.end.private_call_stack,
            array_lengths.private_call_stack - 1 // Do not copy the top item in the stack.
        );
    }

    fn validate_propagated_from_private_call(
        self,
        private_call: PrivateCircuitPublicInputs,
        array_lengths: PrivateCircuitPublicInputsArrayLengths,
        offsets: PrivateKernelCircuitPublicInputsArrayLengths,
        num_popped_call: u32
    ) {
        let storage_contract_address = private_call.call_context.storage_contract_address;
        validate_array_appended_scoped(
            self.output.validation_requests.note_hash_read_requests,
            private_call.note_hash_read_requests,
            array_lengths.note_hash_read_requests,
            offsets.note_hash_read_requests,
            storage_contract_address
        );
        validate_array_appended_scoped(
            self.output.validation_requests.nullifier_read_requests,
            private_call.nullifier_read_requests,
            array_lengths.nullifier_read_requests,
            offsets.nullifier_read_requests,
            storage_contract_address
        );
        validate_array_appended_scoped(
            self.output.validation_requests.scoped_key_validation_requests_and_generators,
            private_call.key_validation_requests_and_generators,
            array_lengths.key_validation_requests_and_generators,
            offsets.scoped_key_validation_requests_and_generators,
            storage_contract_address
        );
        validate_array_appended_scoped(
            self.output.end.note_hashes,
            private_call.note_hashes,
            array_lengths.note_hashes,
            offsets.note_hashes,
            storage_contract_address
        );
        validate_array_appended_scoped(
            self.output.end.nullifiers,
            private_call.nullifiers,
            array_lengths.nullifiers,
            offsets.nullifiers,
            storage_contract_address
        );
        validate_array_appended_scoped(
            self.output.end.l2_to_l1_msgs,
            private_call.l2_to_l1_msgs,
            array_lengths.l2_to_l1_msgs,
            offsets.l2_to_l1_msgs,
            storage_contract_address
        );
        validate_array_appended(
            self.output.end.note_encrypted_logs_hashes,
            private_call.note_encrypted_logs_hashes,
            array_lengths.note_encrypted_logs_hashes,
            offsets.note_encrypted_logs_hashes
        );
        validate_array_appended_scoped(
            self.output.end.encrypted_logs_hashes,
            private_call.encrypted_logs_hashes,
            array_lengths.encrypted_logs_hashes,
            offsets.encrypted_logs_hashes,
            storage_contract_address
        );
        validate_array_appended_scoped(
            self.output.end.unencrypted_logs_hashes,
            private_call.unencrypted_logs_hashes,
            array_lengths.unencrypted_logs_hashes,
            offsets.unencrypted_logs_hashes,
            storage_contract_address
        );
        validate_array_appended(
            self.output.end.public_call_requests,
            private_call.public_call_requests,
            array_lengths.public_call_requests,
            offsets.public_call_requests
        );
        validate_array_appended_reversed(
            self.output.end.private_call_stack,
            private_call.private_call_requests,
            array_lengths.private_call_requests,
            offsets.private_call_stack - num_popped_call
        );
    }
}
