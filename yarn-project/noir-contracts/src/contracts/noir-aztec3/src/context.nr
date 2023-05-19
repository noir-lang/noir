use crate::types::vec::BoundedVec;
use crate::abi;

// Oracles
use crate::oracle::get_l1_to_l2_message::get_l1_to_l2_message_call;

// l1 to l2 messaging
use crate::messaging::l1_to_l2_message_getter_data::L1ToL2MessageGetterData;
use crate::messaging::l1_to_l2_message_getter_data::make_l1_to_l2_message_getter_data;

// When finished, one can call .finish() to convert back to the abi
struct PrivateFunctionContext {
    // inputs: abi::Inputs,
    args : BoundedVec<Field, abi::MAX_ARGS>,
    return_values : BoundedVec<Field, abi::MAX_RETURN_VALUES>,
    logs : BoundedVec<Field, abi::MAX_LOGS>,

    new_commitments: BoundedVec<Field, abi::MAX_NEW_COMMITMENTS>,
    new_nullifiers: BoundedVec<Field, abi::MAX_NEW_NULLIFIERS>,

    private_call_stack : BoundedVec<Field, abi::MAX_PRIVATE_CALL_STACK>,
    public_call_stack : BoundedVec<Field, abi::MAX_PUBLIC_CALL_STACK>,
    new_l2_to_l1_msgs : BoundedVec<Field, abi::MAX_L1_MSG_STACK>,
}

impl PrivateFunctionContext {
    // fn new(inputs: abi::Inputs) -> PrivateFunctionContext {
    fn new() -> PrivateFunctionContext {
        PrivateFunctionContext {
            // inputs: inputs,
            args: BoundedVec::new(0),
            return_values: BoundedVec::new(0),
            logs: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),

            private_call_stack: BoundedVec::new(0),
            public_call_stack: BoundedVec::new(0),
            new_l2_to_l1_msgs: BoundedVec::new(0),
        }
    }

    // When noir uses user-defined serialization of fields, we can return public inputs instead
    fn finish(self, inputs: abi::Inputs) -> [Field; abi::PUBLIC_INPUTS_LENGTH] {
        let priv_circuit_pub_inputs = abi::PrivateCircuitPublicInputs {
            call_context: inputs.call_context,
            args: self.args.storage,
            return_values: self.return_values.storage,
            emitted_events: self.logs.storage,
            new_commitments: self.new_commitments.storage,
            new_nullifiers: self.new_nullifiers.storage,
            private_call_stack: self.private_call_stack.storage,
            public_call_stack: self.public_call_stack.storage,
            new_l2_to_l1_msgs: self.new_l2_to_l1_msgs.storage,
            historic_private_data_tree_root: inputs.old_private_data_tree_root,
            historic_private_nullifier_tree_root: inputs.old_nullifier_tree_root,
            historic_contract_tree_root: inputs.old_contract_tree_root,
            historic_l1_to_l2_messages_tree_root: inputs.old_l1_to_l2_messages_tree_root,
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

    fn message_portal(mut self: Self, msg: Field) -> Self {
        self.new_l2_to_l1_msgs = self.new_l2_to_l1_msgs.push(msg);
        self
    }

    // Inputs must be temporarily passed in to prevent too many unknowns
    // Note this returns self to get around an issue where mutable structs do not maintain mutations unless reassigned
    fn consume_l1_to_l2_message(mut self: Self, inputs: abi::Inputs, msg_key: Field, content_hash: Field, secret: Field) -> PrivateFunctionContext {
        let returned_message = get_l1_to_l2_message_call(msg_key);
        let l1_to_l2_message_data = make_l1_to_l2_message_getter_data(returned_message, 0, secret);

        // Check tree roots against the inputs 
        constrain l1_to_l2_message_data.root == inputs.old_l1_to_l2_messages_tree_root;

        // Validate this is the target contract
        constrain l1_to_l2_message_data.message.recipient == inputs.call_context.storage_contract_address;

        // Validate the message hash is correct
        constrain l1_to_l2_message_data.message.content_hash == content_hash;

        // Validate the message secret is correct
        l1_to_l2_message_data.message.validate_message_secret();

        // Compute Nullifier
        let nullifier = l1_to_l2_message_data.message.compute_nullifier();

        // Push nullifier
        self = self.push_new_nullifier(nullifier);

        self
    }
}