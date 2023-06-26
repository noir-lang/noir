use crate::types::vec::BoundedVec;
use crate::abi;

// l1 to l2 messaging
use crate::messaging::process_l1_to_l2_message;

// When finished, one can call .finish() to convert back to the abi
struct PrivateFunctionContext {
    // inputs: abi::PrivateContextInputs,
    args : BoundedVec<Field, abi::MAX_ARGS>,
    return_values : BoundedVec<Field, abi::MAX_RETURN_VALUES>,

    read_requests: BoundedVec<Field, abi::MAX_READ_REQUESTS>,

    new_commitments: BoundedVec<Field, abi::MAX_NEW_COMMITMENTS>,
    new_nullifiers: BoundedVec<Field, abi::MAX_NEW_NULLIFIERS>,

    private_call_stack : BoundedVec<Field, abi::MAX_PRIVATE_CALL_STACK>,
    public_call_stack : BoundedVec<Field, abi::MAX_PUBLIC_CALL_STACK>,
    new_l2_to_l1_msgs : BoundedVec<Field, abi::MAX_L2_TO_L1_MSGS>,

    // TODO #588, relevant issue: https://github.com/AztecProtocol/aztec-packages/issues/588
    // encrypted_logs_hash : BoundedVec<Field, abi::LOGS_HASH_SIZE>,
    // unencrypted_logs_hash : BoundedVec<Field, abi::LOGS_HASH_SIZE>,
    // encrypted_log_preimages_length : Field,
    // unencrypted_log_preimages_length : Field,
}

impl PrivateFunctionContext {
    // fn new(inputs: abi::PrivateContextInputs) -> PrivateFunctionContext {
    fn new() -> PrivateFunctionContext {
        PrivateFunctionContext {
            // inputs: inputs,
            args: BoundedVec::new(0),
            return_values: BoundedVec::new(0),

            read_requests: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),

            private_call_stack: BoundedVec::new(0),
            public_call_stack: BoundedVec::new(0),
            new_l2_to_l1_msgs: BoundedVec::new(0),

            // TODO #588, relevant issue: https://github.com/AztecProtocol/aztec-packages/issues/588
            // encrypted_logs_hash: BoundedVec::new(0),
            // unencrypted_logs_hash: BoundedVec::new(0),
            // encrypted_log_preimages_length: 0,
            // unencrypted_log_preimages_length: 0,
        }
    }

    fn finish(self, inputs: abi::PrivateContextInputs) -> abi::PrivateCircuitPublicInputs {
        // TODO hash_index
        let args_hash = dep::std::hash::pedersen(self.args.storage)[0];

        let priv_circuit_pub_inputs = abi::PrivateCircuitPublicInputs {
            call_context: inputs.call_context,
            args_hash: args_hash,
            return_values: self.return_values.storage,
            read_requests: self.read_requests.storage,
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            // TODO #588, relevant issue: https://github.com/AztecProtocol/aztec-packages/issues/588
            // encrypted_logs_hash: self.encrypted_logs_hash.storage,
            // unencrypted_logs_hash: self.unencrypted_logs_hash.storage,
            // encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            // unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
            commitment_trees_roots: inputs.roots,
            contract_deployment_data: inputs.contract_deployment_data,
        };
        priv_circuit_pub_inputs
    }

    fn push_read_request(mut self: Self, read_request: Field) -> Self {
        self.read_requests = self.read_requests.push(read_request);
        self
    }

    fn push_new_note_hash(mut self: Self, note_hash: Field) -> Self {
        self.new_commitments = self.new_commitments.push(note_hash);
        self
    }

    fn push_new_nullifier(mut self: Self, nullifier: Field) -> Self {
        self.new_nullifiers = self.new_nullifiers.push(nullifier);
        self
    }

    fn message_portal(mut self: Self, msg: Field) -> Self {
        self.new_l2_to_l1_msgs = self.new_l2_to_l1_msgs.push(msg);
        self
    }

    // PrivateContextInputs must be temporarily passed in to prevent too many unknowns
    // Note this returns self to get around an issue where mutable structs do not maintain mutations unless reassigned
    fn consume_l1_to_l2_message(mut self: Self, inputs: abi::PrivateContextInputs, msg_key: Field, content: Field, secret: Field) -> PrivateFunctionContext {
        let nullifier = process_l1_to_l2_message(inputs.roots.l1_to_l2_messages_tree_root, inputs.call_context.storage_contract_address, msg_key, content, secret);

        // Push nullifier
        self.push_new_nullifier(nullifier)
    }

}
