use crate::types::vec::BoundedVec;
use crate::constants_gen::{
    RETURN_VALUES_LENGTH,
    MAX_READ_REQUESTS_PER_CALL,
    MAX_NEW_COMMITMENTS_PER_CALL,
    MAX_NEW_NULLIFIERS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
    EMPTY_NULLIFIED_COMMITMENT,
    NUM_FIELDS_PER_SHA256,
};
use crate::abi;

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
// use dep::std::collections::vec::Vec;

// l1 to l2 messaging
use crate::messaging::process_l1_to_l2_message;

// When finished, one can call .finish() to convert back to the abi
struct Context {
    inputs: abi::PrivateContextInputs,

    args_hash : Field,
    return_values : BoundedVec<Field, RETURN_VALUES_LENGTH>,

    read_requests: BoundedVec<Field, MAX_READ_REQUESTS_PER_CALL>,

    new_commitments: BoundedVec<Field, MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers: BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL>,
    nullified_commitments: BoundedVec<Field, MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack : BoundedVec<Field, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack : BoundedVec<Field, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_l2_to_l1_msgs : BoundedVec<Field, MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
    // encrypted_logs_preimages: Vec<Field>,
    // unencrypted_logs_preimages: Vec<Field>,
}

impl Context {
    fn new(inputs: abi::PrivateContextInputs, args_hash: Field) -> Context {
        Context {
            inputs: inputs,

            args_hash: args_hash,
            return_values: BoundedVec::new(0),

            read_requests: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),
            nullified_commitments: BoundedVec::new(0),

            private_call_stack: BoundedVec::new(0),
            public_call_stack: BoundedVec::new(0),
            new_l2_to_l1_msgs: BoundedVec::new(0),

            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
            // encrypted_logs_preimages: Vec::new(),
            // unencrypted_logs_preimages: Vec::new(),
        }
    }

    fn finish(self) -> abi::PrivateCircuitPublicInputs {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
        let encrypted_logs_hash = [0; NUM_FIELDS_PER_SHA256];
        let unencrypted_logs_hash = [0; NUM_FIELDS_PER_SHA256];
        let encrypted_log_preimages_length = 0;
        let unencrypted_log_preimages_length = 0;

        let priv_circuit_pub_inputs = abi::PrivateCircuitPublicInputs {
            call_context: self.inputs.call_context,
            args_hash: self.args_hash,
            return_values: self.return_values.storage,
            read_requests: self.read_requests.storage,
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            nullified_commitments: self.nullified_commitments.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            encrypted_logs_hash: encrypted_logs_hash,
            unencrypted_logs_hash: unencrypted_logs_hash,
            encrypted_log_preimages_length: encrypted_log_preimages_length,
            unencrypted_log_preimages_length: unencrypted_log_preimages_length,
            commitment_trees_roots: self.inputs.roots,
            contract_deployment_data: self.inputs.contract_deployment_data,
            chain_id: self.inputs.private_global_variables.chain_id,
            version: self.inputs.private_global_variables.version,
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

    fn push_new_nullifier(mut self: Self, nullifier: Field, nullified_commitment: Field) -> Self {
        self.new_nullifiers = self.new_nullifiers.push(nullifier);
        self.nullified_commitments = self.nullified_commitments.push(nullified_commitment);
        self
    }

    fn message_portal(mut self: Self, msg: Field) -> Self {
        self.new_l2_to_l1_msgs = self.new_l2_to_l1_msgs.push(msg);
        self
    }

    // PrivateContextInputs must be temporarily passed in to prevent too many unknowns
    // Note this returns self to get around an issue where mutable structs do not maintain mutations unless reassigned
    fn consume_l1_to_l2_message(mut self: Self, inputs: abi::PrivateContextInputs, msg_key: Field, content: Field, secret: Field) -> Context {
        let nullifier = process_l1_to_l2_message(inputs.roots.l1_to_l2_messages_tree_root, inputs.call_context.storage_contract_address, msg_key, content, secret);

        // Push nullifier (and the "commitment" corresponding to this can be "empty")
        self.push_new_nullifier(nullifier, EMPTY_NULLIFIED_COMMITMENT)
    }

    fn accumulate_encrypted_logs<N>(mut self: Self, log: [Field; N]) -> Self {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
        self
    }

    fn accumulate_unencrypted_logs<T>(mut self: Self, log: T) -> Self {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165)
        self
    }
}
