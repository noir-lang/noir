use crate::{
    abis::{
    call_context::CallContext, max_block_number::MaxBlockNumber, gas_settings::GasSettings,
    validation_requests::KeyValidationRequestAndGenerator, note_hash::NoteHash, nullifier::Nullifier,
    private_call_request::PrivateCallRequest, public_call_request::PublicCallRequest,
    read_request::ReadRequest, log_hash::{LogHash, NoteLogHash, EncryptedLogHash}
},
    constants::{
    MAX_NOTE_HASH_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_READ_REQUESTS_PER_CALL,
    MAX_KEY_VALIDATION_REQUESTS_PER_CALL, MAX_NOTE_HASHES_PER_CALL, MAX_NULLIFIERS_PER_CALL,
    MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
    MAX_L2_TO_L1_MSGS_PER_CALL, PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH,
    GENERATOR_INDEX__PRIVATE_CIRCUIT_PUBLIC_INPUTS, MAX_ENCRYPTED_LOGS_PER_CALL,
    MAX_UNENCRYPTED_LOGS_PER_CALL, MAX_NOTE_ENCRYPTED_LOGS_PER_CALL
},
    header::Header, hash::pedersen_hash, messaging::l2_to_l1_message::L2ToL1Message,
    traits::{Deserialize, Hash, Serialize, Empty}, utils::reader::Reader,
    transaction::tx_context::TxContext, utils::arrays::validate_array
};

struct PrivateCircuitPublicInputsArrayLengths {
    note_hash_read_requests: u32,
    nullifier_read_requests: u32,
    key_validation_requests_and_generators: u32,
    note_hashes: u32,
    nullifiers: u32,
    l2_to_l1_msgs: u32,
    private_call_requests: u32,
    public_call_requests: u32,
    note_encrypted_logs_hashes: u32,
    encrypted_logs_hashes: u32,
    unencrypted_logs_hashes: u32,
}

impl PrivateCircuitPublicInputsArrayLengths {
    pub fn new(public_inputs: PrivateCircuitPublicInputs) -> Self {
        PrivateCircuitPublicInputsArrayLengths {
            note_hash_read_requests: validate_array(public_inputs.note_hash_read_requests),
            nullifier_read_requests: validate_array(public_inputs.nullifier_read_requests),
            key_validation_requests_and_generators: validate_array(public_inputs.key_validation_requests_and_generators),
            note_hashes: validate_array(public_inputs.note_hashes),
            nullifiers: validate_array(public_inputs.nullifiers),
            l2_to_l1_msgs: validate_array(public_inputs.l2_to_l1_msgs),
            private_call_requests: validate_array(public_inputs.private_call_requests),
            public_call_requests: validate_array(public_inputs.public_call_requests),
            note_encrypted_logs_hashes: validate_array(public_inputs.note_encrypted_logs_hashes),
            encrypted_logs_hashes: validate_array(public_inputs.encrypted_logs_hashes),
            unencrypted_logs_hashes: validate_array(public_inputs.unencrypted_logs_hashes)
        }
    }
}

struct PrivateCircuitPublicInputs {
    call_context: CallContext,

    args_hash: Field,
    returns_hash: Field,

    min_revertible_side_effect_counter: u32,
    is_fee_payer: bool,

    max_block_number: MaxBlockNumber,

    note_hash_read_requests: [ReadRequest; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    nullifier_read_requests: [ReadRequest; MAX_NULLIFIER_READ_REQUESTS_PER_CALL],
    key_validation_requests_and_generators: [KeyValidationRequestAndGenerator; MAX_KEY_VALIDATION_REQUESTS_PER_CALL],

    note_hashes: [NoteHash; MAX_NOTE_HASHES_PER_CALL],
    nullifiers: [Nullifier; MAX_NULLIFIERS_PER_CALL],
    private_call_requests: [PrivateCallRequest; MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
    public_call_requests: [PublicCallRequest; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    public_teardown_call_request: PublicCallRequest,
    l2_to_l1_msgs: [L2ToL1Message; MAX_L2_TO_L1_MSGS_PER_CALL],

    start_side_effect_counter : u32,
    end_side_effect_counter : u32,
    note_encrypted_logs_hashes: [NoteLogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_CALL],
    encrypted_logs_hashes: [EncryptedLogHash; MAX_ENCRYPTED_LOGS_PER_CALL],
    unencrypted_logs_hashes: [LogHash; MAX_UNENCRYPTED_LOGS_PER_CALL],

    // Header of a block whose state is used during private execution (not the block the transaction is included in).
    historical_header: Header,

    // Note: The chain_id and version here are not redundant to the values in self.historical_header.global_variables because
    // they can be different in case of a protocol upgrade. In such a situation we could be using header from a block
    // before the upgrade took place but be using the updated protocol to execute and prove the transaction.
    tx_context: TxContext,
}

impl Eq for PrivateCircuitPublicInputs {
    fn eq(self, other: Self) -> bool {
        self.call_context.eq(other.call_context) &
        self.args_hash.eq(other.args_hash) &
        (self.returns_hash == other.returns_hash) &
        (self.min_revertible_side_effect_counter == other.min_revertible_side_effect_counter) &
        (self.is_fee_payer == other.is_fee_payer) &
        (self.max_block_number == other.max_block_number) &
        (self.note_hash_read_requests == other.note_hash_read_requests) &
        (self.nullifier_read_requests == other.nullifier_read_requests) &
        (self.key_validation_requests_and_generators == other.key_validation_requests_and_generators) &
        (self.note_hashes == other.note_hashes) &
        (self.nullifiers == other.nullifiers) &
        (self.private_call_requests == other.private_call_requests) &
        (self.public_call_requests == other.public_call_requests) &
        (self.l2_to_l1_msgs == other.l2_to_l1_msgs) &
        (self.start_side_effect_counter == other.start_side_effect_counter) &
        (self.end_side_effect_counter == other.end_side_effect_counter) &
        (self.note_encrypted_logs_hashes == other.note_encrypted_logs_hashes) &
        (self.encrypted_logs_hashes == other.encrypted_logs_hashes) &
        (self.unencrypted_logs_hashes == other.unencrypted_logs_hashes) &
        self.historical_header.eq(other.historical_header) &
        self.tx_context.eq(other.tx_context)
    }
}

impl Serialize<PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PrivateCircuitPublicInputs {
    fn serialize(self) -> [Field; PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PRIVATE_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new();
        fields.extend_from_array(self.call_context.serialize());
        fields.push(self.args_hash);
        fields.push(self.returns_hash);

        fields.push(self.min_revertible_side_effect_counter as Field);
        fields.push(if self.is_fee_payer { 1 } else { 0 } as Field);

        fields.extend_from_array(self.max_block_number.serialize());

        for i in 0..self.note_hash_read_requests.len() {
            fields.extend_from_array(self.note_hash_read_requests[i].serialize());
        }
        for i in 0..self.nullifier_read_requests.len() {
            fields.extend_from_array(self.nullifier_read_requests[i].serialize());
        }
        for i in 0..self.key_validation_requests_and_generators.len() {
            fields.extend_from_array(self.key_validation_requests_and_generators[i].serialize());
        }
        for i in 0..self.note_hashes.len() {
            fields.extend_from_array(self.note_hashes[i].serialize());
        }
        for i in 0..self.nullifiers.len() {
            fields.extend_from_array(self.nullifiers[i].serialize());
        }
        for i in 0..self.private_call_requests.len() {
            fields.extend_from_array(self.private_call_requests[i].serialize());
        }
        for i in 0..self.public_call_requests.len() {
            fields.extend_from_array(self.public_call_requests[i].serialize());
        }
        fields.extend_from_array(self.public_teardown_call_request.serialize());
        for i in 0..self.l2_to_l1_msgs.len() {
            fields.extend_from_array(self.l2_to_l1_msgs[i].serialize());
        }
        fields.push(self.start_side_effect_counter as Field);
        fields.push(self.end_side_effect_counter as Field);
        for i in 0..self.note_encrypted_logs_hashes.len() {
            fields.extend_from_array(self.note_encrypted_logs_hashes[i].serialize());
        }
        for i in 0..self.encrypted_logs_hashes.len() {
            fields.extend_from_array(self.encrypted_logs_hashes[i].serialize());
        }
        for i in 0..self.unencrypted_logs_hashes.len() {
            fields.extend_from_array(self.unencrypted_logs_hashes[i].serialize());
        }
        fields.extend_from_array(self.historical_header.serialize());
        fields.extend_from_array(self.tx_context.serialize());

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
            is_fee_payer: reader.read() == 1,
            max_block_number: reader.read_struct(MaxBlockNumber::deserialize),
            note_hash_read_requests: reader.read_struct_array(ReadRequest::deserialize, [ReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL]),
            nullifier_read_requests: reader.read_struct_array(ReadRequest::deserialize, [ReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_CALL]),
            key_validation_requests_and_generators: reader.read_struct_array(KeyValidationRequestAndGenerator::deserialize, [KeyValidationRequestAndGenerator::empty(); MAX_KEY_VALIDATION_REQUESTS_PER_CALL]),
            note_hashes: reader.read_struct_array(NoteHash::deserialize, [NoteHash::empty(); MAX_NOTE_HASHES_PER_CALL]),
            nullifiers: reader.read_struct_array(Nullifier::deserialize, [Nullifier::empty(); MAX_NULLIFIERS_PER_CALL]),
            private_call_requests: reader.read_struct_array(PrivateCallRequest::deserialize, [PrivateCallRequest::empty(); MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL]),
            public_call_requests: reader.read_struct_array(PublicCallRequest::deserialize, [PublicCallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL]),
            public_teardown_call_request: reader.read_struct(PublicCallRequest::deserialize),
            l2_to_l1_msgs: reader.read_struct_array(L2ToL1Message::deserialize, [L2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_CALL]),
            start_side_effect_counter: reader.read() as u32,
            end_side_effect_counter: reader.read() as u32,
            note_encrypted_logs_hashes: reader.read_struct_array(NoteLogHash::deserialize, [NoteLogHash::empty(); MAX_NOTE_ENCRYPTED_LOGS_PER_CALL]),
            encrypted_logs_hashes: reader.read_struct_array(EncryptedLogHash::deserialize, [EncryptedLogHash::empty(); MAX_ENCRYPTED_LOGS_PER_CALL]),
            unencrypted_logs_hashes: reader.read_struct_array(LogHash::deserialize, [LogHash::empty(); MAX_UNENCRYPTED_LOGS_PER_CALL]),
            historical_header: reader.read_struct(Header::deserialize),
            tx_context: reader.read_struct(TxContext::deserialize),
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
            is_fee_payer: false,
            max_block_number: MaxBlockNumber::empty(),
            note_hash_read_requests: [ReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
            nullifier_read_requests: [ReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_CALL],
            key_validation_requests_and_generators: [KeyValidationRequestAndGenerator::empty(); MAX_KEY_VALIDATION_REQUESTS_PER_CALL],
            note_hashes: [NoteHash::empty(); MAX_NOTE_HASHES_PER_CALL],
            nullifiers: [Nullifier::empty(); MAX_NULLIFIERS_PER_CALL],
            private_call_requests: [PrivateCallRequest::empty(); MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL],
            public_call_requests: [PublicCallRequest::empty(); MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
            public_teardown_call_request: PublicCallRequest::empty(),
            l2_to_l1_msgs: [L2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_CALL],
            start_side_effect_counter : 0 as u32,
            end_side_effect_counter : 0 as u32,
            note_encrypted_logs_hashes: [NoteLogHash::empty(); MAX_NOTE_ENCRYPTED_LOGS_PER_CALL],
            encrypted_logs_hashes: [EncryptedLogHash::empty(); MAX_ENCRYPTED_LOGS_PER_CALL],
            unencrypted_logs_hashes: [LogHash::empty(); MAX_UNENCRYPTED_LOGS_PER_CALL],
            historical_header: Header::empty(),
            tx_context: TxContext::empty(),
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

    // Value from private_circuit_public_inputs.test.ts "computes empty item hash" test
    let test_data_empty_hash = 0x2f5d0ff52163d6b688be54324d50d173d9433658bb40e7c5140876df4eb95406;
    assert_eq(hash, test_data_empty_hash);
}
