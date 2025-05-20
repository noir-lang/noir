mod find_first_revertible_item_index;
mod validate_contract_address;
mod validate_split_ranges;

use dep::types::{
    abis::{
        kernel_circuit_public_inputs::PrivateKernelCircuitPublicInputs,
        note_hash::ScopedNoteHash,
        private_call_request::PrivateCallRequest,
        private_circuit_public_inputs::PrivateCircuitPublicInputsArrayLengths,
        private_kernel::private_call_data::PrivateCallData,
        public_call_request::PublicCallRequest,
        side_effect::{Ordered, RangeOrdered},
    },
    address::AztecAddress,
    constants::{
        CONTRACT_CLASS_LOG_SIZE_IN_FIELDS, MAX_FIELD_VALUE, PRIVATE_LOG_SIZE_IN_FIELDS,
        REGISTERER_CONTRACT_ADDRESS,
    },
    traits::FromField,
    transaction::tx_request::TxRequest,
    utils::arrays::find_index_hint,
};
use find_first_revertible_item_index::find_first_revertible_item_index;
use validate_contract_address::validate_contract_address;
use validate_split_ranges::validate_split_ranges;

fn validate_incrementing_counters_within_range<T, let N: u32>(
    counter_start: u32,
    counter_end: u32,
    items: [T; N],
    num_items: u32,
)
where
    T: Ordered,
{
    let mut prev_counter = counter_start;
    let mut should_check = true;
    for i in 0..N {
        should_check &= i != num_items;
        if should_check {
            let item = items[i];
            assert(
                item.counter() > prev_counter,
                "counter must be larger than the counter of the previous item",
            );
            prev_counter = item.counter();
        }
    }
    assert(prev_counter < counter_end, "counter must be smaller than the end counter of the call");
}

fn validate_incrementing_counter_ranges_within_range<T, let N: u32>(
    counter_start: u32,
    counter_end: u32,
    items: [T; N],
    num_items: u32,
)
where
    T: RangeOrdered,
{
    let mut prev_counter = counter_start;
    let mut should_check = true;
    for i in 0..N {
        should_check &= i != num_items;
        if should_check {
            let item = items[i];
            assert(
                item.counter_start() > prev_counter,
                "start counter must be larger than the end counter of the previous call",
            );
            assert(
                item.counter_end() > item.counter_start(),
                "nested call has incorrect counter range",
            );
            prev_counter = item.counter_end();
        }
    }
    assert(
        prev_counter < counter_end,
        "end counter must be smaller than the end counter of the parent call",
    );
}

pub struct PrivateCallDataValidator {
    data: PrivateCallData,
    pub array_lengths: PrivateCircuitPublicInputsArrayLengths,
}

impl PrivateCallDataValidator {
    pub fn new(data: PrivateCallData) -> Self {
        let array_lengths = PrivateCircuitPublicInputsArrayLengths::new(data.public_inputs);
        PrivateCallDataValidator { data, array_lengths }
    }

    pub fn validate<let N: u32>(
        self,
        accumulated_note_hashes: [ScopedNoteHash; N],
        protocol_contract_tree_root: Field,
    ) {
        validate_contract_address(self.data, protocol_contract_tree_root);
        self.validate_call();
        self.validate_private_call_requests();
        self.validate_public_call_requests();
        self.validate_counters();
        self.validate_logs(accumulated_note_hashes);
        self.validate_log_lengths();
    }

    pub fn validate_as_first_call(self) {
        let public_inputs = self.data.public_inputs;
        let call_context = public_inputs.call_context;
        assert(call_context.is_static_call == false, "Users cannot make a static call");
        assert(
            call_context.msg_sender == AztecAddress::from_field(MAX_FIELD_VALUE),
            "Users cannot set msg_sender in first call",
        );
    }

    // Confirm that the TxRequest (user's intent) matches the private call being executed.
    pub fn validate_against_tx_request(self, tx_request: TxRequest) {
        let public_inputs = self.data.public_inputs;
        assert_eq(
            tx_request.origin,
            public_inputs.call_context.contract_address,
            "contract address does not match origin",
        );
        assert_eq(
            tx_request.function_data.selector,
            public_inputs.call_context.function_selector,
            "function_selector in call_context does not match the value in tx_request",
        );
        assert(
            tx_request.function_data.is_private,
            "tx_request does not indicate the first function is private",
        );
        assert_eq(
            tx_request.args_hash,
            public_inputs.args_hash,
            "args_hash in private call does not match the value in tx_request",
        );
        assert_eq(
            tx_request.tx_context,
            public_inputs.tx_context,
            "tx_context in private call does not match the value in tx_request",
        );
    }

    pub fn validate_against_call_request(self, request: PrivateCallRequest) {
        let public_inputs = self.data.public_inputs;
        assert_eq(
            request.call_context,
            public_inputs.call_context,
            "call_context does not match call request",
        );
        assert_eq(
            request.args_hash,
            public_inputs.args_hash,
            "args_hash does not match call request",
        );
        assert_eq(
            request.returns_hash,
            public_inputs.returns_hash,
            "returns_hash does not match call request",
        );
        assert_eq(
            request.start_side_effect_counter,
            public_inputs.start_side_effect_counter,
            "start_side_effect_counter does not match call request",
        );
        assert_eq(
            request.end_side_effect_counter,
            public_inputs.end_side_effect_counter,
            "end_side_effect_counter does not match call request",
        );
    }

    pub fn validate_against_previous_kernel(
        self,
        previous_kernel: PrivateKernelCircuitPublicInputs,
    ) {
        let constants = previous_kernel.constants;
        let public_inputs = self.data.public_inputs;
        assert_eq(
            public_inputs.historical_header,
            constants.historical_header,
            "mismatch historical header",
        );
        assert_eq(public_inputs.tx_context, constants.tx_context, "mismatch tx context");
        // constants.global_variables is not relevant to private functions and is ensured to be empty in PrivateKernelCircuitOutputValidator.
    }

    fn validate_call(self) {
        let call_context = self.data.public_inputs.call_context;
        if call_context.is_static_call {
            // No state changes are allowed for static calls:
            assert_eq(
                self.array_lengths.note_hashes,
                0,
                "note_hashes must be empty for static calls",
            );
            assert_eq(
                self.array_lengths.nullifiers,
                0,
                "nullifiers must be empty for static calls",
            );
            assert_eq(
                self.array_lengths.l2_to_l1_msgs,
                0,
                "l2_to_l1_msgs must be empty for static calls",
            );
            assert_eq(
                self.array_lengths.private_logs,
                0,
                "private_logs must be empty for static calls",
            );
            assert_eq(
                self.array_lengths.contract_class_logs_hashes,
                0,
                "contract_class_logs_hashes must be empty for static calls",
            );
        }
        if (self.array_lengths.contract_class_logs_hashes != 0) {
            assert_eq(
                call_context.contract_address,
                REGISTERER_CONTRACT_ADDRESS,
                "only the class registerer may emit contract class logs",
            );
        }
    }

    fn validate_private_call_requests(self) {
        let public_inputs = self.data.public_inputs;
        let call_requests = public_inputs.private_call_requests;
        let num_requests = self.array_lengths.private_call_requests;
        let mut should_check = true;
        for i in 0..call_requests.len() {
            should_check &= i != num_requests;
            if should_check {
                let call_request = call_requests[i];
                assert_eq(
                    call_request.call_context.msg_sender,
                    public_inputs.call_context.contract_address,
                    "incorrect msg_sender for call request",
                );
                if !call_request.call_context.is_static_call {
                    assert(
                        public_inputs.call_context.is_static_call == false,
                        "static call cannot make non-static calls",
                    );
                }
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
        // Safety: the index is constrained in validate_split_ranges below.
        let first_revertible_index = unsafe {
            find_first_revertible_item_index(
                public_inputs.min_revertible_side_effect_counter,
                public_inputs.private_call_requests,
            )
        };
        validate_split_ranges(
            min_revertible_side_effect_counter,
            first_revertible_index,
            public_inputs.private_call_requests,
            self.array_lengths.private_call_requests,
        );
    }

    fn validate_public_call_requests(self) {
        let public_inputs = self.data.public_inputs;

        let call_requests = public_inputs.public_call_requests;
        let num_requests = self.array_lengths.public_call_requests;
        let mut should_check = true;
        for i in 0..call_requests.len() {
            should_check &= i != num_requests;
            if should_check {
                self.validate_public_call_request(call_requests[i].inner);
            }
        }

        if !public_inputs.public_teardown_call_request.contract_address.is_zero() {
            self.validate_public_call_request(public_inputs.public_teardown_call_request);
        }
    }

    fn validate_public_call_request(self, call_request: PublicCallRequest) {
        let this_context = self.data.public_inputs.call_context;
        assert_eq(
            call_request.msg_sender,
            this_context.contract_address,
            "incorrect msg_sender for call request",
        );
        if !call_request.is_static_call {
            assert(
                this_context.is_static_call == false,
                "static call cannot make non-static calls",
            );
        }
    }

    fn validate_counters(self) {
        let public_inputs = self.data.public_inputs;
        let counter_start = public_inputs.start_side_effect_counter;
        let counter_end = public_inputs.end_side_effect_counter;

        assert(counter_start < counter_end, "private call has incorrect counter range");

        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.note_hash_read_requests,
            self.array_lengths.note_hash_read_requests,
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.nullifier_read_requests,
            self.array_lengths.nullifier_read_requests,
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.note_hashes,
            self.array_lengths.note_hashes,
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.nullifiers,
            self.array_lengths.nullifiers,
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.l2_to_l1_msgs,
            self.array_lengths.l2_to_l1_msgs,
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.private_logs,
            self.array_lengths.private_logs,
        );
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.contract_class_logs_hashes,
            self.array_lengths.contract_class_logs_hashes,
        );
        validate_incrementing_counter_ranges_within_range(
            counter_start,
            counter_end,
            public_inputs.private_call_requests,
            self.array_lengths.private_call_requests,
        );

        // Validate the public call requests by checking their start counters only, as their end counters are unknown.
        validate_incrementing_counters_within_range(
            counter_start,
            counter_end,
            public_inputs.public_call_requests,
            self.array_lengths.public_call_requests,
        );
    }

    fn validate_logs<let N: u32>(self, accumulated_note_hashes: [ScopedNoteHash; N]) {
        let logs = self.data.public_inputs.private_logs;
        let contract_address = self.data.public_inputs.call_context.contract_address;
        for i in 0..logs.len() {
            let log = logs[i];
            if log.note_hash_counter != 0 {
                // Safety: the index is constrained by the checks below.
                let note_index = unsafe {
                    find_index_hint(
                        accumulated_note_hashes,
                        |n: ScopedNoteHash| n.counter() == log.note_hash_counter,
                    )
                };
                assert(note_index != N, "could not find note hash linked to note log");
                let note_hash = accumulated_note_hashes[note_index];
                assert_eq(
                    log.note_hash_counter,
                    note_hash.counter(),
                    "could not find note hash linked to note log",
                );
                assert_eq(
                    contract_address,
                    note_hash.contract_address,
                    "could not link a note log to a note hash in another contract",
                );
            }
        }
    }

    fn validate_log_lengths(self) {
        let public_inputs = self.data.public_inputs;

        // Private logs.
        let logs = public_inputs.private_logs;
        for i in 0..logs.len() {
            let log = logs[i];
            assert(log.log.length <= PRIVATE_LOG_SIZE_IN_FIELDS, "private log length is too large");
        }

        // Contract class logs.
        let logs = public_inputs.contract_class_logs_hashes;
        for i in 0..logs.len() {
            let log = logs[i];
            assert(
                log.inner.length <= CONTRACT_CLASS_LOG_SIZE_IN_FIELDS,
                "contract class log length is too large",
            );
        }
    }
}
