use crate::types::vec::BoundedVec;
use crate::abi;

// When finished, one can call .finish() to convert back to the abi
struct PrivateFunctionContext {
    return_values : BoundedVec<Field, abi::MAX_RETURN_VALUES>,
    logs : BoundedVec<Field, abi::MAX_LOGS>,

    new_commitments: BoundedVec<Field, abi::MAX_NEW_COMMITMENTS>,
    new_nullifiers: BoundedVec<Field, abi::MAX_NEW_NULLIFIERS>,

    private_call_stack : BoundedVec<Field, abi::MAX_PRIVATE_CALL_STACK>,
    public_call_stack : BoundedVec<Field, abi::MAX_PUBLIC_CALL_STACK>,
    l1_message_stack : BoundedVec<Field, abi::MAX_L1_MSG_STACK>,
}

impl PrivateFunctionContext {
    fn new() -> PrivateFunctionContext {
        PrivateFunctionContext {
            return_values: BoundedVec::new(0),
            logs: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),

            private_call_stack: BoundedVec::new(0),
            public_call_stack: BoundedVec::new(0),
            l1_message_stack: BoundedVec::new(0),
        }
    }

    // When noir uses user-defined serialization of fields, we can return public inputs instead
    fn finish(self, inputs: abi::Inputs) -> [Field; abi::PUBLIC_INPUTS_LENGTH] {
        let priv_circuit_pub_inputs = abi::PrivateCircuitPublicInputs {
            call_context: inputs.call_context,
            args: inputs.args,
            return_values: self.return_values.storage,
            emitted_events: self.logs.storage,
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            l1_msg_stack: self.l1_message_stack.storage,
            historic_private_data_tree_root: inputs.old_private_data_tree_root,
            historic_private_nullifier_tree_root: inputs.old_nullifier_tree_root,
            historic_contract_tree_root: inputs.old_contract_tree_root,
            contract_deployment_data: inputs.contract_deployment_data,
        };
        priv_circuit_pub_inputs.serialize()
    }

    fn push_new_note_hash(mut self: Self, note_hash: Field) -> Self {
        self.new_commitments = self.new_commitments.push(note_hash);
        self
    }

    fn push_new_nullifier(mut self: Self, nullifier: Field) -> Self {
        self.new_nullifiers = self.new_nullifiers.push(nullifier);
        self
    }
}