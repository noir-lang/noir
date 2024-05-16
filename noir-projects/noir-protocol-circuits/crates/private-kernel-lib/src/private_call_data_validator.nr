use dep::types::{
    abis::{
    call_context::CallContext, call_request::CallRequest, private_call_stack_item::PrivateCallStackItem,
    private_kernel::private_call_data::PrivateCallData, side_effect::Ordered
},
    address::{AztecAddress, PartialAddress}, contract_class_id::ContractClassId,
    hash::{private_functions_root_from_siblings, stdlib_recursion_verification_key_compress_native_vk},
    traits::{is_empty, is_empty_array}, transaction::tx_request::TxRequest,
    utils::arrays::{array_length, validate_array}
};

fn validate_arrays(data: PrivateCallData) -> ArrayLengths {
    let public_inputs = data.call_stack_item.public_inputs;

    // Each of the following arrays is expected to be zero-padded.
    ArrayLengths {
        note_hash_read_requests: validate_array(public_inputs.note_hash_read_requests),
        nullifier_read_requests: validate_array(public_inputs.nullifier_read_requests),
        nullifier_key_validation_requests: validate_array(public_inputs.nullifier_key_validation_requests),
        new_note_hashes: validate_array(public_inputs.new_note_hashes),
        new_nullifiers: validate_array(public_inputs.new_nullifiers),
        new_l2_to_l1_msgs: validate_array(public_inputs.new_l2_to_l1_msgs),
        private_call_stack_hashes: validate_array(public_inputs.private_call_stack_hashes),
        public_call_stack_hashes: validate_array(public_inputs.public_call_stack_hashes),
        note_encrypted_logs_hashes: validate_array(public_inputs.note_encrypted_logs_hashes),
        encrypted_logs_hashes: validate_array(public_inputs.encrypted_logs_hashes),
        unencrypted_logs_hashes: validate_array(public_inputs.unencrypted_logs_hashes)
    }
}

fn is_valid_caller(request: CallRequest, caller_address: AztecAddress, caller_context: CallContext) -> bool {
    let valid_caller_context = request.caller_context.msg_sender.eq(caller_context.msg_sender)
        & request.caller_context.storage_contract_address.eq(caller_context.storage_contract_address);

    request.caller_contract_address.eq(caller_address)
        & (request.caller_context.is_empty() | valid_caller_context)
}

fn validate_call_request(request: CallRequest, hash: Field, caller: PrivateCallStackItem) {
    if hash != 0 {
        assert_eq(request.hash, hash, "call stack hash does not match call request hash");
        assert(
            is_valid_caller(
                request,
                caller.contract_address,
                caller.public_inputs.call_context
            ), "invalid caller"
        );
    } else {
        assert(is_empty(request), "call requests length does not match the expected length");
    }
}

fn validate_incrementing_counters_within_range<T, N>(
    counter_start: u32,
    counter_end: u32,
    items: [T; N],
    num_items: u64
) where T: Ordered {
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

fn validate_incrementing_counter_ranges_within_range<N>(
    counter_start: u32,
    counter_end: u32,
    items: [CallRequest; N],
    num_items: u64
) {
    let mut prev_counter = counter_start;
    let mut should_check = true;
    for i in 0..N {
        should_check &= i != num_items;
        if should_check {
            let item = items[i];
            assert(
                item.start_side_effect_counter > prev_counter, "start counter must be larger than the end counter of the previous call"
            );
            assert(
                item.end_side_effect_counter > item.start_side_effect_counter, "nested call has incorrect counter range"
            );
            prev_counter = item.end_side_effect_counter;
        }
    }
    assert(
        prev_counter < counter_end, "end counter must be smaller than the end counter of the parent call"
    );
}

fn validate_split_private_call_requests<N>(
    min_revertible_side_effect_counter: u32,
    first_revertible_call_request_index: u64,
    call_requests: [CallRequest; N],
    num_call_requests: u64
) {
    if first_revertible_call_request_index != 0 {
        let last_non_revertible_call_request_index = first_revertible_call_request_index - 1;
        let call_request = call_requests[last_non_revertible_call_request_index];
        assert(
            min_revertible_side_effect_counter > call_request.end_side_effect_counter, "min_revertible_side_effect_counter must be greater than the end counter of the last non revertible call"
        );
    }
    if first_revertible_call_request_index != num_call_requests {
        let call_request = call_requests[first_revertible_call_request_index];
        assert(
            min_revertible_side_effect_counter <= call_request.start_side_effect_counter, "min_revertible_side_effect_counter must be less than or equal to the start counter of the first revertible call"
        );
    }
}

fn validate_split_public_call_requests<N>(
    min_revertible_side_effect_counter: u32,
    first_revertible_call_request_index: u64,
    call_requests: [CallRequest; N],
    num_call_requests: u64
) {
    if first_revertible_call_request_index != 0 {
        let last_non_revertible_call_request_index = first_revertible_call_request_index - 1;
        let call_request = call_requests[last_non_revertible_call_request_index];
        assert(
            min_revertible_side_effect_counter > call_request.counter(), "min_revertible_side_effect_counter must be greater than the counter of the last non revertible call"
        );
    }
    if first_revertible_call_request_index != num_call_requests {
        let call_request = call_requests[first_revertible_call_request_index];
        assert(
            min_revertible_side_effect_counter <= call_request.counter(), "min_revertible_side_effect_counter must be less than or equal to the counter of the first revertible call"
        );
    }
}

struct ArrayLengths {
    note_hash_read_requests: u64,
    nullifier_read_requests: u64,
    nullifier_key_validation_requests: u64,
    new_note_hashes: u64,
    new_nullifiers: u64,
    new_l2_to_l1_msgs: u64,
    private_call_stack_hashes: u64,
    public_call_stack_hashes: u64,
    note_encrypted_logs_hashes: u64,
    encrypted_logs_hashes: u64,
    unencrypted_logs_hashes: u64,
}

struct PrivateCallDataValidator {
    data: PrivateCallData,
    array_lengths: ArrayLengths,
}

impl PrivateCallDataValidator {
    pub fn new(data: PrivateCallData) -> Self {
        let array_lengths = validate_arrays(data);
        PrivateCallDataValidator { data, array_lengths }
    }

    pub fn validate(self) {
        self.validate_contract_address();
        self.validate_call();
        self.validate_private_call_requests();
        self.validate_public_call_requests();
        self.validate_teardown_call_request();
        self.validate_counters();
    }

    pub fn validate_as_first_call(
        self,
        first_revertible_private_call_request_index: u64,
        first_revertible_public_call_request_index: u64
    ) {
        let public_inputs = self.data.call_stack_item.public_inputs;
        let call_context = public_inputs.call_context;
        assert(call_context.is_delegate_call == false, "Users cannot make a delegatecall");
        assert(call_context.is_static_call == false, "Users cannot make a static call");

        let min_revertible_side_effect_counter = public_inputs.min_revertible_side_effect_counter;
        // No need to check that the min_revertible_side_effect_counter falls in the counter range of the private call.
        // It is valid as long as it does not fall in the middle of any nested call.
        validate_split_private_call_requests(
            min_revertible_side_effect_counter,
            first_revertible_private_call_request_index,
            self.data.private_call_stack,
            self.array_lengths.private_call_stack_hashes
        );
        validate_split_public_call_requests(
            min_revertible_side_effect_counter,
            first_revertible_public_call_request_index,
            self.data.public_call_stack,
            self.array_lengths.public_call_stack_hashes
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

    pub fn validate_against_call_request(self, request: CallRequest) {
        let call_stack_item = self.data.call_stack_item;

        assert_eq(
            request.hash, call_stack_item.hash(), "calculated private_call_hash does not match provided private_call_hash at the top of the call stack"
        );

        let call_context = call_stack_item.public_inputs.call_context;

        if call_context.is_delegate_call {
            let caller_context = request.caller_context;
            assert(!caller_context.is_empty(), "caller context cannot be empty for delegate calls");
            assert_eq(
                call_context.msg_sender, caller_context.msg_sender, "call stack msg_sender does not match expected msg_sender for delegate calls"
            );
            assert_eq(
                call_context.storage_contract_address, caller_context.storage_contract_address, "call stack storage address does not match expected contract address for delegate calls"
            );
        } else {
            assert_eq(
                call_context.msg_sender, request.caller_contract_address, "call stack msg_sender does not match caller contract address"
            );
        }
    }

    fn validate_contract_address(self) {
        let contract_address = self.data.call_stack_item.contract_address;

        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3062): Why is this using a hash function from the stdlib::recursion namespace
        let private_call_vk_hash = stdlib_recursion_verification_key_compress_native_vk(self.data.vk);

        assert(!contract_address.is_zero(), "contract address cannot be zero");
        // std::println(f"contract_address={contract_address}");
        // std::println(f"private_call_vk_hash={private_call_vk_hash}");

        // Recompute the contract class id
        let computed_private_functions_root = private_functions_root_from_siblings(
            self.data.call_stack_item.function_data.selector,
            private_call_vk_hash,
            self.data.function_leaf_membership_witness.leaf_index,
            self.data.function_leaf_membership_witness.sibling_path
        );
        // std::println(f"computed_private_functions_root={computed_private_functions_root}");

        let computed_contract_class_id = ContractClassId::compute(
            self.data.contract_class_artifact_hash,
            computed_private_functions_root,
            self.data.contract_class_public_bytecode_commitment
        );
        // std::println(f"computed_contract_class_id={computed_contract_class_id}");

        // Recompute contract address using the preimage which includes the class_id
        let computed_partial_address = PartialAddress::compute_from_salted_initialization_hash(
            computed_contract_class_id,
            self.data.salted_initialization_hash
        );
        // std::println(f"computed_partial_address={computed_partial_address}");

        let computed_address = AztecAddress::compute(self.data.public_keys_hash, computed_partial_address);
        // std::println(f"computed_address={computed_address}");

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

        if call_context.is_static_call {
            // No state changes are allowed for static calls:
            assert_eq(self.array_lengths.new_note_hashes, 0, "new_note_hashes must be empty for static calls");
            assert_eq(self.array_lengths.new_nullifiers, 0, "new_nullifiers must be empty for static calls");
            assert_eq(
                self.array_lengths.new_l2_to_l1_msgs, 0, "new_l2_to_l1_msgs must be empty for static calls"
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
        let call_requests = self.data.private_call_stack;
        let hashes = self.data.call_stack_item.public_inputs.private_call_stack_hashes;
        for i in 0..call_requests.len() {
            validate_call_request(call_requests[i], hashes[i], self.data.call_stack_item);
        }
    }

    fn validate_public_call_requests(self) {
        let call_requests = self.data.public_call_stack;
        let hashes = self.data.call_stack_item.public_inputs.public_call_stack_hashes;
        for i in 0..call_requests.len() {
            validate_call_request(call_requests[i], hashes[i], self.data.call_stack_item);
        }
    }

    fn validate_teardown_call_request(self) {
        validate_call_request(
            self.data.public_teardown_call_request,
            self.data.call_stack_item.public_inputs.public_teardown_function_hash,
            self.data.call_stack_item
        );
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
            public_inputs.new_note_hashes,
            self.array_lengths.new_note_hashes
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.new_nullifiers,
            self.array_lengths.new_nullifiers
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.new_l2_to_l1_msgs,
            self.array_lengths.new_l2_to_l1_msgs
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
            self.data.private_call_stack,
            self.array_lengths.private_call_stack_hashes
        );

        // Validate the public call requests by checking their start counters only, as their end counters are unknown.
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            self.data.public_call_stack,
            self.array_lengths.public_call_stack_hashes
        );

        let teardown_call_request_count = if self.data.public_teardown_call_request.hash == 0 {
            0
        } else {
            1
        };
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            [self.data.public_teardown_call_request],
            teardown_call_request_count
        );
    }
}
