mod find_first_revertible_item_index;
mod validate_split_ranges;

use crate::components::private_call_data_validator::{
    find_first_revertible_item_index::find_first_revertible_item_index,
    validate_split_ranges::validate_split_ranges
};
use dep::types::{
    abis::{
    call_context::CallContext, kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
    log_hash::NoteLogHash, note_hash::ScopedNoteHash, private_call_request::PrivateCallRequest,
    private_circuit_public_inputs::{PrivateCircuitPublicInputs, PrivateCircuitPublicInputsArrayLengths},
    private_kernel::private_call_data::PrivateCallData, side_effect::{Ordered, RangeOrdered}
},
    address::{AztecAddress, PartialAddress}, contract_class_id::ContractClassId,
    constants::MAX_FIELD_VALUE,
    hash::{private_functions_root_from_siblings, stdlib_recursion_verification_key_compress_native_vk},
    traits::is_empty, transaction::tx_request::TxRequest, utils::arrays::find_index
};

unconstrained fn match_log_to_note<N>(
    note_log: NoteLogHash,
    accumulated_note_hashes: [ScopedNoteHash; N]
) -> u32 {
    find_index(
        accumulated_note_hashes,
        |n: ScopedNoteHash| n.counter() == note_log.note_hash_counter
    )
}

unconstrained fn find_first_revertible_private_call_request_index(public_inputs: PrivateCircuitPublicInputs) -> u32 {
    find_first_revertible_item_index(
        public_inputs.min_revertible_side_effect_counter,
        public_inputs.private_call_requests
    )
}

fn validate_call_context(
    target_contract: AztecAddress,
    target_context: CallContext,
    this_context: CallContext
) {
    if target_context.is_delegate_call {
        assert_eq(
            target_context.msg_sender, this_context.msg_sender, "incorrect msg_sender for delegate call request"
        );
        assert_eq(
            target_context.storage_contract_address, this_context.storage_contract_address, "incorrect storage_contract_address for delegate call request"
        );
    } else {
        assert_eq(
            target_context.msg_sender, this_context.storage_contract_address, "incorrect msg_sender for call request"
        );
        assert_eq(
            target_context.storage_contract_address, target_contract, "incorrect storage_contract_address for call request"
        );
    }
    if !target_context.is_static_call {
        assert(this_context.is_static_call == false, "static call cannot make non-static calls");
    }
}

fn validate_incrementing_counters_within_range<T, N>(counter_start: u32, counter_end: u32, items: [T; N], num_items: u32) where T: Ordered {
    let mut prev_counter = counter_start;
    let mut should_check = true;
    for i in 0..N {
        should_check &= i != num_items;
        if should_check {
            let item = items[i];
            assert(
                item.counter() > prev_counter, "counter must be larger than the counter of the previous item"
            );
            prev_counter = item.counter();
        }
    }
    assert(prev_counter < counter_end, "counter must be smaller than the end counter of the call");
}

fn validate_incrementing_counter_ranges_within_range<T, N>(
    counter_start: u32,
    counter_end: u32,
    items: [T; N],
    num_items: u32
) where T: RangeOrdered {
    let mut prev_counter = counter_start;
    let mut should_check = true;
    for i in 0..N {
        should_check &= i != num_items;
        if should_check {
            let item = items[i];
            assert(
                item.counter_start() > prev_counter, "start counter must be larger than the end counter of the previous call"
            );
            assert(item.counter_end() > item.counter_start(), "nested call has incorrect counter range");
            prev_counter = item.counter_end();
        }
    }
    assert(
        prev_counter < counter_end, "end counter must be smaller than the end counter of the parent call"
    );
}

struct PrivateCallDataValidator {
    data: PrivateCallData,
    array_lengths: PrivateCircuitPublicInputsArrayLengths,
}

impl PrivateCallDataValidator {
    pub fn new(data: PrivateCallData) -> Self {
        let array_lengths = PrivateCircuitPublicInputsArrayLengths::new(data.call_stack_item.public_inputs);
        PrivateCallDataValidator { data, array_lengths }
    }

    pub fn validate<N>(self, accumulated_note_hashes: [ScopedNoteHash; N]) {
        self.validate_contract_address();
        self.validate_call();
        self.validate_private_call_requests();
        self.validate_public_call_requests();
        self.validate_teardown_call_request();
        self.validate_counters();
        self.validate_note_logs(accumulated_note_hashes);
    }

    pub fn validate_as_first_call(self) {
        let public_inputs = self.data.call_stack_item.public_inputs;
        let call_context = public_inputs.call_context;
        assert(call_context.is_delegate_call == false, "Users cannot make a delegatecall");
        assert(call_context.is_static_call == false, "Users cannot make a static call");
        assert(
            call_context.msg_sender == AztecAddress::from_field(MAX_FIELD_VALUE), "Users cannot set msg_sender in first call"
        );
    }

    // Confirm that the TxRequest (user's intent) matches the private call being executed.
    pub fn validate_against_tx_request(self, tx_request: TxRequest) {
        let call_stack_item = self.data.call_stack_item;
        assert_eq(
            tx_request.origin, call_stack_item.contract_address, "origin address does not match call stack items contract address"
        );
        assert_eq(
            tx_request.function_data.hash(), call_stack_item.function_data.hash(), "tx_request function_data must match call_stack_item function_data"
        );
        assert_eq(
            tx_request.args_hash, call_stack_item.public_inputs.args_hash, "noir function args passed to tx_request must match args in the call_stack_item"
        );
        assert_eq(
            tx_request.tx_context, call_stack_item.public_inputs.tx_context, "tx_context in tx_request must match tx_context in call_stack_item"
        );
    }

    pub fn validate_against_call_request(self, request: PrivateCallRequest) {
        let call_stack_item = self.data.call_stack_item;

        assert_eq(
            request.contract_address, call_stack_item.contract_address, "contract_address does not match call request"
        );
        assert_eq(
            request.call_context, call_stack_item.public_inputs.call_context, "call_context does not match call request"
        );
        assert_eq(
            request.args_hash, call_stack_item.public_inputs.args_hash, "args_hash does not match call request"
        );
        assert_eq(
            request.returns_hash, call_stack_item.public_inputs.returns_hash, "returns_hash does not match call request"
        );
        assert_eq(
            request.start_side_effect_counter, call_stack_item.public_inputs.start_side_effect_counter, "start_side_effect_counter does not match call request"
        );
        assert_eq(
            request.end_side_effect_counter, call_stack_item.public_inputs.end_side_effect_counter, "end_side_effect_counter does not match call request"
        );
    }

    pub fn validate_against_previous_kernel(self, previous_kernel: PrivateKernelCircuitPublicInputs) {
        let constants = previous_kernel.constants;
        let public_inputs = self.data.call_stack_item.public_inputs;
        assert_eq(public_inputs.historical_header, constants.historical_header, "mismatch historical header");
        assert_eq(public_inputs.tx_context, constants.tx_context, "mismatch tx context");
        // constants.global_variables is not relevant to private functions and is ensured to be empty in PrivateKernelCircuitOutputValidator.
    }

    fn validate_contract_address(self) {
        let contract_address = self.data.call_stack_item.contract_address;

        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3062): Why is this using a hash function from the stdlib::recursion namespace
        let private_call_vk_hash = stdlib_recursion_verification_key_compress_native_vk(self.data.vk);

        assert(!contract_address.is_zero(), "contract address cannot be zero");
        // println(f"contract_address={contract_address}");
        // println(f"private_call_vk_hash={private_call_vk_hash}");

        // Recompute the contract class id
        let computed_private_functions_root = private_functions_root_from_siblings(
            self.data.call_stack_item.function_data.selector,
            private_call_vk_hash,
            self.data.function_leaf_membership_witness.leaf_index,
            self.data.function_leaf_membership_witness.sibling_path
        );
        // println(f"computed_private_functions_root={computed_private_functions_root}");

        let computed_contract_class_id = ContractClassId::compute(
            self.data.contract_class_artifact_hash,
            computed_private_functions_root,
            self.data.contract_class_public_bytecode_commitment
        );
        // println(f"computed_contract_class_id={computed_contract_class_id}");

        // Recompute contract address using the preimage which includes the class_id
        let computed_partial_address = PartialAddress::compute_from_salted_initialization_hash(
            computed_contract_class_id,
            self.data.salted_initialization_hash
        );
        // println(f"computed_partial_address={computed_partial_address}");

        let computed_address = AztecAddress::compute(self.data.public_keys_hash, computed_partial_address);
        // println(f"computed_address={computed_address}");

        assert(
            computed_address.eq(contract_address), "computed contract address does not match expected one"
        );
    }

    fn validate_call(self) {
        let call_context = self.data.call_stack_item.public_inputs.call_context;

        let is_own_storage = call_context.storage_contract_address == self.data.call_stack_item.contract_address;
        if call_context.is_delegate_call {
            assert(
                !is_own_storage, "current contract address must not match storage contract address for delegate calls"
            );
        } else {
            assert(is_own_storage, "call stack storage address does not match expected contract address");
        }

        assert_eq(
            call_context.function_selector, self.data.call_stack_item.function_data.selector, "function selector in call context does not match call stack item"
        );

        if call_context.is_static_call {
            // No state changes are allowed for static calls:
            assert_eq(self.array_lengths.note_hashes, 0, "note_hashes must be empty for static calls");
            assert_eq(self.array_lengths.nullifiers, 0, "nullifiers must be empty for static calls");
            assert_eq(self.array_lengths.l2_to_l1_msgs, 0, "l2_to_l1_msgs must be empty for static calls");
            assert_eq(
                self.array_lengths.note_encrypted_logs_hashes, 0, "note_encrypted_logs_hashes must be empty for static calls"
            );
            assert_eq(
                self.array_lengths.encrypted_logs_hashes, 0, "encrypted_logs_hashes must be empty for static calls"
            );
            assert_eq(
                self.array_lengths.unencrypted_logs_hashes, 0, "unencrypted_logs_hashes must be empty for static calls"
            );
        }
    }

    fn validate_private_call_requests(self) {
        let public_inputs = self.data.call_stack_item.public_inputs;
        let call_requests = public_inputs.private_call_requests;
        let num_requests = self.array_lengths.private_call_requests;
        let mut should_check = true;
        for i in 0..call_requests.len() {
            should_check &= i != num_requests;
            if should_check {
                validate_call_context(
                    call_requests[i].contract_address,
                    call_requests[i].call_context,
                    public_inputs.call_context
                );
            }
        }

        // Check that the min_revertible_side_effect_counter does not fall in the middle of any nested calls.
        // If it is possible, one can create a custom account contract, set the min_revertible_side_effect_counter to a
        // value that falls in a transfer function, make the tx revert which then preserves the note hashes and discards 
        // the nullifiers.
        // 
        // We don't have to use the aggregated min_revertible_side_effect_counter in the output for the below check.
        // If the current call's min_revertible_side_effect_counter is 0, it means the counter might be set by another
        // function. This check for that function guarantees that the counter won't fall into one of its nested calls. 
        // In this case, we can simply use 0 and make the following check pass.
        let min_revertible_side_effect_counter = public_inputs.min_revertible_side_effect_counter;
        let first_revertible_index = find_first_revertible_private_call_request_index(public_inputs);
        validate_split_ranges(
            min_revertible_side_effect_counter,
            first_revertible_index,
            public_inputs.private_call_requests,
            self.array_lengths.private_call_requests
        );
    }

    fn validate_public_call_requests(self) {
        let public_inputs = self.data.call_stack_item.public_inputs;
        let call_requests = self.data.call_stack_item.public_inputs.public_call_requests;
        let num_requests = self.array_lengths.public_call_requests;
        let mut should_check = true;
        for i in 0..call_requests.len() {
            should_check &= i != num_requests;
            if should_check {
                validate_call_context(
                    call_requests[i].item.contract_address,
                    call_requests[i].item.call_context,
                    public_inputs.call_context
                );
            }
        }
    }

    fn validate_teardown_call_request(self) {
        let public_inputs = self.data.call_stack_item.public_inputs;
        let request = self.data.call_stack_item.public_inputs.public_teardown_call_request;
        if request.counter != 0 {
            validate_call_context(
                request.item.contract_address,
                request.item.call_context,
                public_inputs.call_context
            );
        }
    }

    fn validate_counters(self) {
        let public_inputs = self.data.call_stack_item.public_inputs;
        let counter_start = public_inputs.start_side_effect_counter;
        let counter_end = public_inputs.end_side_effect_counter;

        assert(counter_start < counter_end, "private call has incorrect counter range");

        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.note_hash_read_requests,
            self.array_lengths.note_hash_read_requests
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.nullifier_read_requests,
            self.array_lengths.nullifier_read_requests
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.note_hashes,
            self.array_lengths.note_hashes
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.nullifiers,
            self.array_lengths.nullifiers
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.l2_to_l1_msgs,
            self.array_lengths.l2_to_l1_msgs
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.encrypted_logs_hashes,
            self.array_lengths.encrypted_logs_hashes
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.unencrypted_logs_hashes,
            self.array_lengths.unencrypted_logs_hashes
        );
        validate_incrementing_counter_ranges_within_range(
            counter_start,
            counter_end,
            public_inputs.private_call_requests,
            self.array_lengths.private_call_requests
        );

        // Validate the public call requests by checking their start counters only, as their end counters are unknown.
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.public_call_requests,
            self.array_lengths.public_call_requests
        );

        let teardown_call_request_count = if public_inputs.public_teardown_call_request.counter == 0 {
            0
        } else {
            1
        };
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            [public_inputs.public_teardown_call_request],
            teardown_call_request_count
        );
    }

    fn validate_note_logs<N>(self, accumulated_note_hashes: [ScopedNoteHash; N]) {
        let note_logs = self.data.call_stack_item.public_inputs.note_encrypted_logs_hashes;
        let num_logs = self.array_lengths.note_encrypted_logs_hashes;
        let storage_contract_address = self.data.call_stack_item.public_inputs.call_context.storage_contract_address;
        let mut should_check = true;
        for i in 0..note_logs.len() {
            should_check &= i != num_logs;
            if should_check {
                let note_index = match_log_to_note(note_logs[i], accumulated_note_hashes);
                assert(note_index != N, "could not find note hash linked to note log");
                assert_eq(
                    note_logs[i].note_hash_counter, accumulated_note_hashes[note_index].counter(), "could not find note hash linked to note log"
                );
                // If the note_index points to an empty note hash, the following check will fail.
                assert_eq(
                    accumulated_note_hashes[note_index].contract_address, storage_contract_address, "could not link a note log to a note hash in another contract"
                );
            }
        }
    }
}
