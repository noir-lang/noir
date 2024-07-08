use crate::{
    abis::{
    call_context::CallContext, note_hash::NoteHash, nullifier::Nullifier, read_request::ReadRequest,
    gas::Gas, global_variables::GlobalVariables, log_hash::LogHash
},
    address::AztecAddress,
    constants::{
    MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL, MAX_L2_TO_L1_MSGS_PER_CALL, MAX_NULLIFIERS_PER_CALL,
    MAX_NOTE_HASHES_PER_CALL, MAX_NOTE_HASH_READ_REQUESTS_PER_CALL,
    MAX_NULLIFIER_READ_REQUESTS_PER_CALL, MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL, MAX_PUBLIC_DATA_READS_PER_CALL,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL, GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS,
    PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH, MAX_UNENCRYPTED_LOGS_PER_CALL
},
    contrakt::{storage_read::StorageRead, storage_update_request::StorageUpdateRequest},
    hash::pedersen_hash, header::Header, messaging::l2_to_l1_message::L2ToL1Message,
    traits::{Hash, Serialize, Deserialize, Empty}, utils::reader::Reader
};

struct PublicCircuitPublicInputs {
    call_context: CallContext,

    args_hash: Field,
    returns_hash: Field,

    note_hash_read_requests: [ReadRequest; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    nullifier_read_requests: [ReadRequest; MAX_NULLIFIER_READ_REQUESTS_PER_CALL],
    nullifier_non_existent_read_requests: [ReadRequest; MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL],
    l1_to_l2_msg_read_requests: [ReadRequest; MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL],
    contract_storage_update_requests: [StorageUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL],
    contract_storage_reads: [StorageRead; MAX_PUBLIC_DATA_READS_PER_CALL],

    // todo: add sideeffect ranges for the input to these hashes
    public_call_stack_hashes: [Field; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
    note_hashes: [NoteHash; MAX_NOTE_HASHES_PER_CALL],
    nullifiers: [Nullifier; MAX_NULLIFIERS_PER_CALL],
    l2_to_l1_msgs: [L2ToL1Message; MAX_L2_TO_L1_MSGS_PER_CALL],

    start_side_effect_counter: u32,
    end_side_effect_counter: u32,

    unencrypted_logs_hashes: [LogHash; MAX_UNENCRYPTED_LOGS_PER_CALL],

    // Header of a block whose state is used during public execution. Set by sequencer to be a header of a block
    // previous to the one in which the tx is included.
    historical_header: Header,

    // Global variables injected into this circuit
    global_variables: GlobalVariables,

    prover_address: AztecAddress,

    revert_code: u8,
    
    start_gas_left: Gas,
    end_gas_left: Gas,
    transaction_fee: Field,
}

impl Eq for PublicCircuitPublicInputs {
    fn eq(self, other: Self) -> bool {
        self.serialize() == other.serialize()
    }
}

impl Serialize<PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PublicCircuitPublicInputs {
    fn serialize(self) -> [Field; PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH] {
        let mut fields: BoundedVec<Field, PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH> = BoundedVec::new();
        fields.extend_from_array(self.call_context.serialize());
        fields.push(self.args_hash);
        fields.push(self.returns_hash);
        for i in 0..MAX_NOTE_HASH_READ_REQUESTS_PER_CALL {
            fields.extend_from_array(self.note_hash_read_requests[i].serialize());
        }
        for i in 0..MAX_NULLIFIER_READ_REQUESTS_PER_CALL {
            fields.extend_from_array(self.nullifier_read_requests[i].serialize());
        }
        for i in 0..MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL {
            fields.extend_from_array(self.nullifier_non_existent_read_requests[i].serialize());
        }
        for i in 0..MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL {
            fields.extend_from_array(self.l1_to_l2_msg_read_requests[i].serialize());
        }
        for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL {
            fields.extend_from_array(self.contract_storage_update_requests[i].serialize());
        }
        for i in 0..MAX_PUBLIC_DATA_READS_PER_CALL {
            fields.extend_from_array(self.contract_storage_reads[i].serialize());
        }
        fields.extend_from_array(self.public_call_stack_hashes);

        for i in 0..MAX_NOTE_HASHES_PER_CALL {
            fields.extend_from_array(self.note_hashes[i].serialize());
        }
        for i in 0..MAX_NULLIFIERS_PER_CALL {
            fields.extend_from_array(self.nullifiers[i].serialize());
        }
        for i in 0..MAX_L2_TO_L1_MSGS_PER_CALL {
            fields.extend_from_array(self.l2_to_l1_msgs[i].serialize());
        }

        fields.push(self.start_side_effect_counter as Field);
        fields.push(self.end_side_effect_counter as Field);

        for i in 0..MAX_UNENCRYPTED_LOGS_PER_CALL{
            fields.extend_from_array(self.unencrypted_logs_hashes[i].serialize());
        }
        fields.extend_from_array(self.historical_header.serialize());
        fields.extend_from_array(self.global_variables.serialize());
        fields.push(self.prover_address.to_field());
        fields.push(self.revert_code as Field);
        fields.extend_from_array(self.start_gas_left.serialize());
        fields.extend_from_array(self.end_gas_left.serialize());
        fields.push(self.transaction_fee);
        fields.storage
    }
}

impl Deserialize<PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH> for PublicCircuitPublicInputs {
    fn deserialize(serialized: [Field; PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH]) -> Self {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut reader = Reader::new(serialized);
        let inputs = PublicCircuitPublicInputs {
            call_context: reader.read_struct(CallContext::deserialize),
            args_hash: reader.read(),
            returns_hash: reader.read(),
            note_hash_read_requests: reader.read_struct_array(ReadRequest::deserialize, [ReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL]),
            nullifier_read_requests: reader.read_struct_array(ReadRequest::deserialize, [ReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_CALL]),
            nullifier_non_existent_read_requests: reader.read_struct_array(ReadRequest::deserialize, [ReadRequest::empty(); MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL]),
            l1_to_l2_msg_read_requests: reader.read_struct_array(ReadRequest::deserialize, [ReadRequest::empty(); MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL]),
            contract_storage_update_requests: reader.read_struct_array(StorageUpdateRequest::deserialize, [StorageUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL]),
            contract_storage_reads: reader.read_struct_array(StorageRead::deserialize, [StorageRead::empty(); MAX_PUBLIC_DATA_READS_PER_CALL]),
            public_call_stack_hashes: reader.read_array([0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL]),
            note_hashes: reader.read_struct_array(NoteHash::deserialize, [NoteHash::empty(); MAX_NOTE_HASHES_PER_CALL]),
            nullifiers: reader.read_struct_array(Nullifier::deserialize, [Nullifier::empty(); MAX_NULLIFIERS_PER_CALL]),
            l2_to_l1_msgs: reader.read_struct_array(L2ToL1Message::deserialize, [L2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_CALL]),
            start_side_effect_counter: reader.read() as u32,
            end_side_effect_counter: reader.read() as u32,
            unencrypted_logs_hashes: reader.read_struct_array(LogHash::deserialize, [LogHash::empty(); MAX_UNENCRYPTED_LOGS_PER_CALL]),
            historical_header: reader.read_struct(Header::deserialize),
            global_variables: reader.read_struct(GlobalVariables::deserialize),
            prover_address: reader.read_struct(AztecAddress::deserialize),
            revert_code: reader.read() as u8,
            start_gas_left: reader.read_struct(Gas::deserialize),
            end_gas_left: reader.read_struct(Gas::deserialize),
            transaction_fee: reader.read(),
        };

        reader.finish();
        inputs
    }
}

impl Hash for PublicCircuitPublicInputs {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__PUBLIC_CIRCUIT_PUBLIC_INPUTS)
    }
}

impl Empty for PublicCircuitPublicInputs {
    fn empty() -> Self {
        PublicCircuitPublicInputs {
            call_context: CallContext::empty(),
            args_hash: 0,
            returns_hash: 0,
            note_hash_read_requests: [ReadRequest::empty(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
            nullifier_read_requests: [ReadRequest::empty(); MAX_NULLIFIER_READ_REQUESTS_PER_CALL],
            nullifier_non_existent_read_requests: [ReadRequest::empty(); MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL],
            l1_to_l2_msg_read_requests: [ReadRequest::empty(); MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL],
            contract_storage_update_requests: [StorageUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL],
            contract_storage_reads: [StorageRead::empty(); MAX_PUBLIC_DATA_READS_PER_CALL],
            public_call_stack_hashes: [0; MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL],
            note_hashes: [NoteHash::empty(); MAX_NOTE_HASHES_PER_CALL],
            nullifiers: [Nullifier::empty(); MAX_NULLIFIERS_PER_CALL],
            l2_to_l1_msgs: [L2ToL1Message::empty(); MAX_L2_TO_L1_MSGS_PER_CALL],
            start_side_effect_counter: 0 as u32,
            end_side_effect_counter: 0 as u32,
            unencrypted_logs_hashes: [LogHash::empty(); MAX_UNENCRYPTED_LOGS_PER_CALL],
            historical_header: Header::empty(),
            global_variables: GlobalVariables::empty(),
            prover_address: AztecAddress::zero(),
            revert_code: 0 as u8,
            start_gas_left: Gas::empty(),
            end_gas_left: Gas::empty(),
            transaction_fee: 0,
        }
    }
}

#[test]
fn serialization_of_empty() {
    let pcpi = PublicCircuitPublicInputs::empty();
    let serialized = pcpi.serialize();
    let deserialized = PublicCircuitPublicInputs::deserialize(serialized);
    assert(pcpi.eq(deserialized));
}

#[test]
fn empty_hash() {
    let inputs = PublicCircuitPublicInputs::empty();
    let hash = inputs.hash();

    // Value from public_circuit_public_inputs.test.ts "computes empty item hash" test
    let test_data_empty_hash = 0x2b80e3db92cdbf588155e0eaa4f51b35366cbd6e10f587c335204be5bf337bac;
    assert_eq(hash, test_data_empty_hash);
}
