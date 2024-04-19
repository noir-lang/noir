use crate::{
    abis::{
    call_context::CallContext, max_block_number::MaxBlockNumber,
    nullifier_key_validation_request::NullifierKeyValidationRequest, read_request::ReadRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
},
    constants::{
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL, MAX_NEW_NOTE_HASHES_PER_CALL,
    MAX_NEW_NULLIFIERS_PER_CALL, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
    PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH,
    GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS, MAX_ENCRYPTED_LOGS_PER_CALL, MAX_UNENCRYPTED_LOGS_PER_CALL
},
    header::Header, hash::pedersen_hash, messaging::l2_to_l1_message::L2ToL1Message,
    traits::{Deserialize, Hash, Serialize, Empty}, utils::reader::Reader
};

struct PrivateCircuitPublicInputs {
    call_context: CallContext,

    args_hash: Field,
    returns_hash: Field,

    min_revertible_side_effect_counter: u32,

    max_block_number: MaxBlockNumber,

    note_hash_read_requests: [SideEffect; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    nullifier_read_requests: [ReadRequest; MAX_NULLIFIER_READ_REQUESTS_PER_CALL],
    nullifier_key_validation_requests: [NullifierKeyValidationRequest; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL],

    new_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_CALL],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_CALL],
    private_call_stack_hashes: [Field; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_stack_hashes: [Field; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    new_l2_to_l1_msgs: [L2ToL1Message; MAX_NEW_L2_TO_L1_MSGS_PER_CALL],

    start_side_effect_counter : u32,
    end_side_effect_counter : u32,
    encrypted_logs_hashes: [SideEffect; MAX_ENCRYPTED_LOGS_PER_CALL],
    unencrypted_logs_hashes: [SideEffect; MAX_UNENCRYPTED_LOGS_PER_CALL],

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    // Header of a block whose state is used during private execution (not the block the transaction is included in).
    historical_header: Header,

    // Note: The following 2 values are not redundant to the values in self.historical_header.global_variables because
    // they can be different in case of a protocol upgrade. In such a situation we could be using header from a block
    // before the upgrade took place but be using the updated protocol to execute and prove the transaction.
    chain_id: Field,
    version: Field,
}

impl Eq for PrivateCircuitPublicInputs {
    fn eq(self, other: Self) -> bool {
        self.call_context.eq(other.call_context) &
        self.args_hash.eq(other.args_hash) &
        (self.returns_hash == other.returns_hash) &
        (self.max_block_number == other.max_block_number) &
        (self.note_hash_read_requests == other.note_hash_read_requests) &
        (self.nullifier_read_requests == other.nullifier_read_requests) &
        (self.nullifier_key_validation_requests == other.nullifier_key_validation_requests) &
        (self.new_note_hashes == other.new_note_hashes) &
        (self.new_nullifiers == other.new_nullifiers) &
        (self.private_call_stack_hashes == other.private_call_stack_hashes) &
        (self.public_call_stack_hashes == other.public_call_stack_hashes) &
        (self.new_l2_to_l1_msgs == other.new_l2_to_l1_msgs) &
        (self.start_side_effect_counter == other.start_side_effect_counter) &
        (self.end_side_effect_counter == other.end_side_effect_counter) &
        (self.encrypted_logs_hashes == other.encrypted_logs_hashes) &
        (self.unencrypted_logs_hashes == other.unencrypted_logs_hashes) &
        (self.encrypted_log_preimages_length == other.encrypted_log_preimages_length) &
        (self.unencrypted_log_preimages_length == other.unencrypted_log_preimages_length) &
        self.historical_header.eq(other.historical_header) &
        self.chain_id.eq(other.chain_id) &
        self.version.eq(other.version)
    }
}

impl Serialize<PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateCircuitPublicInputs {
    fn serialize(self) -> [Field; PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new();
        fields.extend_from_array(self.call_context.serialize());
        fields.push(self.args_hash);
        fields.push(self.returns_hash);

        fields.push(self.min_revertible_side_effect_counter as Field);

        fields.extend_from_array(self.max_block_number.serialize());

        for i in 0..self.note_hash_read_requests.len() {
            fields.extend_from_array(self.note_hash_read_requests[i].serialize());
        }
        for i in 0..self.nullifier_read_requests.len() {
            fields.extend_from_array(self.nullifier_read_requests[i].serialize());
        }
        for i in 0..self.nullifier_key_validation_requests.len() {
            fields.extend_from_array(self.nullifier_key_validation_requests[i].serialize());
        }
        for i in 0..self.new_note_hashes.len() {
            fields.extend_from_array(self.new_note_hashes[i].serialize());
        }
        for i in 0..self.new_nullifiers.len() {
            fields.extend_from_array(self.new_nullifiers[i].serialize());
        }
        fields.extend_from_array(self.private_call_stack_hashes);
        fields.extend_from_array(self.public_call_stack_hashes);
        for i in 0..self.new_l2_to_l1_msgs.len() {
            fields.extend_from_array(self.new_l2_to_l1_msgs[i].serialize());
        }
        fields.push(self.start_side_effect_counter as Field);
        fields.push(self.end_side_effect_counter as Field);
        for i in 0..self.encrypted_logs_hashes.len() {
            fields.extend_from_array(self.encrypted_logs_hashes[i].serialize());
        }
        for i in 0..self.unencrypted_logs_hashes.len() {
            fields.extend_from_array(self.unencrypted_logs_hashes[i].serialize());
        }
        fields.push(self.encrypted_log_preimages_length);
        fields.push(self.unencrypted_log_preimages_length);
        fields.extend_from_array(self.historical_header.serialize());
        fields.push(self.chain_id);
        fields.push(self.version);

        assert_eq(fields.len(), PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH);

        fields.storage
    }
}

impl Deserialize<PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateCircuitPublicInputs {
    fn deserialize(serialized: [Field; PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH]) -> Self {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut reader = Reader::new(serialized);
        let inputs = Self {
            call_context: reader.read_struct(CallContext::deserialize),
            args_hash: reader.read(),
            returns_hash: reader.read(),
            min_revertible_side_effect_counter: reader.read() as u32,
            max_block_number: reader.read_struct(MaxBlockNumber::deserialize),
            note_hash_read_requests: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL]),
            nullifier_read_requests: reader.read_struct_array(ReadRequest::deserialize, [ReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_CALL]),
            nullifier_key_validation_requests: reader.read_struct_array(NullifierKeyValidationRequest::deserialize, [NullifierKeyValidationRequest::empty(); MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL]),
            new_note_hashes: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_NEW_NOTE_HASHES_PER_CALL]),
            new_nullifiers: reader.read_struct_array(SideEffectLinkedToNoteHash::deserialize, [SideEffectLinkedToNoteHash::empty(); MAX_NEW_NULLIFIERS_PER_CALL]),
            private_call_stack_hashes: reader.read_array([0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL]),
            public_call_stack_hashes: reader.read_array([0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL]),
            new_l2_to_l1_msgs: reader.read_struct_array(L2ToL1Message::deserialize, [L2ToL1Message::empty(); MAX_NEW_L2_TO_L1_MSGS_PER_CALL]),
            start_side_effect_counter: reader.read() as u32,
            end_side_effect_counter: reader.read() as u32,
            encrypted_logs_hashes: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_ENCRYPTED_LOGS_PER_CALL]),
            unencrypted_logs_hashes: reader.read_struct_array(SideEffect::deserialize, [SideEffect::empty(); MAX_UNENCRYPTED_LOGS_PER_CALL]),
            encrypted_log_preimages_length: reader.read(),
            unencrypted_log_preimages_length: reader.read(),
            historical_header: reader.read_struct(Header::deserialize),
            chain_id: reader.read(),
            version: reader.read(),
        };

        reader.finish();
        inputs
    }
}

impl Hash for PrivateCircuitPublicInputs {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS)
    }
}


impl Empty for PrivateCircuitPublicInputs {
    fn empty() -> Self {
        PrivateCircuitPublicInputs {
            call_context: CallContext::empty(),
            args_hash: 0,
            returns_hash: 0,
            min_revertible_side_effect_counter: 0 as u32,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: [SideEffect::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
            nullifier_read_requests: [ReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_CALL],
            nullifier_key_validation_requests: [NullifierKeyValidationRequest::empty(); MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL],
            new_note_hashes: [SideEffect::empty(); MAX_NEW_NOTE_HASHES_PER_CALL],
            new_nullifiers: [SideEffectLinkedToNoteHash::empty(); MAX_NEW_NULLIFIERS_PER_CALL],
            private_call_stack_hashes: [0; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
            public_call_stack_hashes: [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
            new_l2_to_l1_msgs: [L2ToL1Message::empty(); MAX_NEW_L2_TO_L1_MSGS_PER_CALL],
            start_side_effect_counter : 0 as u32,
            end_side_effect_counter : 0 as u32,
            encrypted_logs_hashes: [SideEffect::empty(); MAX_ENCRYPTED_LOGS_PER_CALL],
            unencrypted_logs_hashes: [SideEffect::empty(); MAX_UNENCRYPTED_LOGS_PER_CALL],
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            historical_header: Header::empty(),
            chain_id: 0,
            version: 0,
        }
    }
}

#[test]
fn serialization_of_empty() {
    let pcpi = PrivateCircuitPublicInputs::empty();
    let serialized = pcpi.serialize();
    let deserialized = PrivateCircuitPublicInputs::deserialize(serialized);
    assert(pcpi.eq(deserialized));
}

#[test]
fn empty_hash() {
    let inputs = PrivateCircuitPublicInputs::empty();
    let hash = inputs.hash();
    // Value from private_circuit_public_inputs.test.ts "computes empty inputs hash" test
    let test_data_empty_hash = 0x265dcde577695a65f00acc24d861e65e9486526ec2b9f9bab9493476f73e4bc9;
    assert_eq(hash, test_data_empty_hash);
}
