global MAX_ARGS: comptime Field = 8;
global MAX_RETURN_VALUES: comptime Field = 4;
global MAX_LOGS: comptime Field = 4;
global MAX_NEW_COMMITMENTS: comptime Field = 4;
global MAX_NEW_NULLIFIERS: comptime Field = 4;
global MAX_PRIVATE_CALL_STACK : comptime Field = 4;
global MAX_PUBLIC_CALL_STACK: comptime Field = 4;
global MAX_L1_MSG_STACK : comptime Field = 2;
global PUBLIC_INPUTS_LENGTH : comptime Field = 47;

use crate::notes::value_note::Note;
use crate::types::vec::BoundedVec;

// Inputs are expected to be provided to each contract function
struct Inputs {
    call_context : CallContext,

    old_private_data_tree_root : Field,
    old_nullifier_tree_root : Field,
    old_contract_tree_root : Field,

    contract_deployment_data: ContractDeploymentData,
    args: [Field; MAX_ARGS],
}

struct CallContext {
    msg_sender : Field,
    storage_contract_address : Field,
    portal_contract_address : Field,

    is_delegate_call : bool,
    is_static_call : bool,
    is_contract_deployment: bool,
}

impl CallContext {
    fn serialize(self) -> [Field; 6] {
        [
            self.msg_sender,
            self.storage_contract_address,
            self.portal_contract_address,
            self.is_delegate_call as Field,
            self.is_static_call as Field,
            self.is_contract_deployment as Field,
        ]
    }
}

struct ContractDeploymentData {
    constructor_vk_hash : Field,
    function_tree_root : Field,
    contract_address_salt : Field,
    portal_contract_address : Field,
}

impl ContractDeploymentData {
    fn serialize(self) -> [Field; 4] {
        [
            self.constructor_vk_hash,
            self.function_tree_root,
            self.contract_address_salt,
            self.portal_contract_address,
        ]
    }
}

struct PrivateCircuitPublicInputs {
    call_context: CallContext,
    args: [Field; crate::abi::MAX_ARGS],
    return_values: [Field; crate::abi::MAX_RETURN_VALUES],
    emitted_events: [Field; crate::abi::MAX_LOGS],
    new_commitments: [Field; crate::abi::MAX_NEW_COMMITMENTS],
    new_nullifiers: [Field; crate::abi::MAX_NEW_NULLIFIERS],
    private_call_stack: [Field; crate::abi::MAX_PRIVATE_CALL_STACK],
    public_call_stack: [Field; crate::abi::MAX_PUBLIC_CALL_STACK],
    l1_msg_stack: [Field; crate::abi::MAX_L1_MSG_STACK],
    historic_private_data_tree_root: Field,
    historic_private_nullifier_tree_root: Field,
    historic_contract_tree_root: Field,
    contract_deployment_data: ContractDeploymentData,
}

impl PrivateCircuitPublicInputs {
    // TODO hash_index
    fn hash(self) -> Field {
        dep::std::hash::pedersen(self.serialize())[0]
    }

    fn serialize(self) -> [Field; PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PUBLIC_INPUTS_LENGTH> = BoundedVec::new(0); 
        fields = fields.push_array(self.call_context.serialize());   
        fields = fields.push_array(self.args);
        fields = fields.push_array(self.return_values);
        fields = fields.push_array(self.emitted_events);
        fields = fields.push_array(self.new_commitments);
        fields = fields.push_array(self.new_nullifiers);
        fields = fields.push_array(self.private_call_stack);
        fields = fields.push_array(self.public_call_stack);
        fields = fields.push_array(self.l1_msg_stack);
        fields = fields.push(self.historic_private_data_tree_root);
        fields = fields.push(self.historic_private_nullifier_tree_root);
        fields = fields.push(self.historic_contract_tree_root);
        fields = fields.push_array(self.contract_deployment_data.serialize());
        fields.storage
    }
}

// When finished, one can call .finish() to convert back to the abi
struct Outputs {
    return_values : BoundedVec<Field, MAX_RETURN_VALUES>,
    logs : BoundedVec<Field, MAX_LOGS>,

    new_commitments: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_nullifiers: BoundedVec<Field, MAX_NEW_NULLIFIERS>,

    private_call_stack : BoundedVec<Field, MAX_PRIVATE_CALL_STACK>,
    public_call_stack : BoundedVec<Field, MAX_PUBLIC_CALL_STACK>,
    l1_message_stack : BoundedVec<Field, MAX_L1_MSG_STACK>,

    new_note_values: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_owner_xs: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_owner_ys: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_randomness: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_nonce: BoundedVec<Field, MAX_NEW_COMMITMENTS>,
    new_note_is_realsies: BoundedVec<bool, MAX_NEW_COMMITMENTS>,

    nullified_note_values: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_owner_xs: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_owner_ys: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_randomness: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_nonce: BoundedVec<Field, MAX_NEW_NULLIFIERS>,
    nullified_note_is_realsies: BoundedVec<bool, MAX_NEW_NULLIFIERS>,
}

impl Outputs {
    fn new() -> Outputs {
        Outputs {
            return_values: BoundedVec::new(0),
            logs: BoundedVec::new(0),

            new_commitments: BoundedVec::new(0),
            new_nullifiers: BoundedVec::new(0),

            private_call_stack: BoundedVec::new(0),
            public_call_stack: BoundedVec::new(0),
            l1_message_stack: BoundedVec::new(0),

            new_note_values: BoundedVec::new(0),
            new_note_owner_xs: BoundedVec::new(0),
            new_note_owner_ys: BoundedVec::new(0),
            new_note_randomness: BoundedVec::new(0),
            new_note_nonce: BoundedVec::new(0),
            new_note_is_realsies: BoundedVec::new(false),

            nullified_note_values: BoundedVec::new(0),
            nullified_note_owner_xs: BoundedVec::new(0),
            nullified_note_owner_ys: BoundedVec::new(0),
            nullified_note_randomness: BoundedVec::new(0),
            nullified_note_nonce: BoundedVec::new(0),
            nullified_note_is_realsies: BoundedVec::new(false),
        }
    }

    // When noir uses user-defined serialization of fields, we can return public inputs instead
    fn finish(self, inputs: Inputs) -> [Field; PUBLIC_INPUTS_LENGTH] {
        let priv_circuit_pub_inputs = PrivateCircuitPublicInputs {
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

    fn push_new_note(mut self: Self, note: Note) -> Self {
        self.new_note_values = self.new_note_values.push(note.value);
        self.new_note_owner_xs = self.new_note_owner_xs.push(note.owner.x);
        self.new_note_owner_ys = self.new_note_owner_ys.push(note.owner.y);
        self.new_note_randomness = self.new_note_randomness.push(note.randomness);
        self.new_note_nonce = self.new_note_nonce.push(note.nonce);
        self.new_note_is_realsies = self.new_note_is_realsies.push(note.is_real);
        self
    }

    fn push_nullified_note(mut self: Self, note: Note) -> Self {
        self.nullified_note_values = self.nullified_note_values.push(note.value);
        self.nullified_note_owner_xs = self.nullified_note_owner_xs.push(note.owner.x);
        self.nullified_note_owner_ys = self.nullified_note_owner_ys.push(note.owner.y);
        self.nullified_note_randomness = self.nullified_note_randomness.push(note.randomness);
        self.nullified_note_nonce = self.nullified_note_nonce.push(note.nonce);
        self.nullified_note_is_realsies = self.nullified_note_is_realsies.push(note.is_real);
        self
    }
}
