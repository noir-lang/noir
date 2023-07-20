use crate::types::vec::BoundedVec;
use crate::constants_gen;
use crate::abi;

// l1 to l2 messaging
use crate::messaging::process_l1_to_l2_message;

// When finished, one can call .finish() to convert back to the abi
struct Context {
    inputs: abi::PrivateContextInputs,
    // TODO(https://github.com/noir-lang/noir/pull/1729): remove when stepwise execution of acir is complete
    oracle_connector: Field,  // for forcing dependencies between oracle calls

    args_hash : Field,
    return_values : BoundedVec<Field, constants_gen::RETURN_VALUES_LENGTH>,

    read_requests: BoundedVec<Field, constants_gen::MAX_READ_REQUESTS_PER_CALL>,

    new_commitments: BoundedVec<Field, constants_gen::MAX_NEW_COMMITMENTS_PER_CALL>,
    new_nullifiers: BoundedVec<Field, constants_gen::MAX_NEW_NULLIFIERS_PER_CALL>,
    nullified_commitments: BoundedVec<Field, constants_gen::MAX_NEW_NULLIFIERS_PER_CALL>,

    private_call_stack : BoundedVec<Field, constants_gen::MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>,
    public_call_stack : BoundedVec<Field, constants_gen::MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    new_l2_to_l1_msgs : BoundedVec<Field, constants_gen::MAX_NEW_L2_TO_L1_MSGS_PER_CALL>,

    encrypted_logs_hash : [Field; constants_gen::NUM_FIELDS_PER_SHA256],
    unencrypted_logs_hash : [Field; constants_gen::NUM_FIELDS_PER_SHA256],
    encrypted_log_preimages_length : Field,
    unencrypted_log_preimages_length : Field,
}

impl Context {
    fn new(inputs: abi::PrivateContextInputs, args_hash: Field) -> Context {
        Context {
            inputs: inputs,
            // TODO(https://github.com/noir-lang/noir/pull/1729): remove when stepwise execution of acir is complete
            oracle_connector: 0,

            args_hash: args_hash,
            return_values: BoundedVec::new(0),

            read_requests: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),
            nullified_commitments: BoundedVec::new(0),

            private_call_stack: BoundedVec::new(0),
            public_call_stack: BoundedVec::new(0),
            new_l2_to_l1_msgs: BoundedVec::new(0),

            encrypted_logs_hash: [0; constants_gen::NUM_FIELDS_PER_SHA256],
            unencrypted_logs_hash: [0; constants_gen::NUM_FIELDS_PER_SHA256],
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
        }
    }

    fn finish(self) -> abi::PrivateCircuitPublicInputs {
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
            encrypted_logs_hash: self.encrypted_logs_hash,
            unencrypted_logs_hash: self.unencrypted_logs_hash,
            encrypted_log_preimages_length: self.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: self.unencrypted_log_preimages_length,
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
        self.push_new_nullifier(nullifier, constants_gen::EMPTY_NULLIFIED_COMMITMENT)
    }

    fn set_encrypted_logs_hash(mut self: Self, hash: [Field; constants_gen::NUM_FIELDS_PER_SHA256]) -> Self {
        self.encrypted_logs_hash = hash;
        self
    }

    fn set_encrypted_log_preimages_length(mut self: Self, len: Field) -> Self {
        self.encrypted_log_preimages_length = len;
        self
    }

    fn set_unencrypted_logs_hash(mut self: Self, hash: [Field; constants_gen::NUM_FIELDS_PER_SHA256]) -> Self {
        self.unencrypted_logs_hash = hash;
        self
    }

    fn set_unencrypted_log_preimages_length(mut self: Self, len: Field) -> Self {
        self.unencrypted_log_preimages_length = len;
        self
    }
}
