use crate::{
    abis::{
    gas::Gas, accumulated_data::public_accumulated_data::PublicAccumulatedData,
    call_request::CallRequest, note_hash::NoteHash, nullifier::Nullifier,
    public_data_update_request::PublicDataUpdateRequest, log_hash::{LogHash, NoteLogHash, ScopedLogHash}
},
    constants::{
    MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    MAX_L2_TO_L1_MSGS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_ENCRYPTED_LOGS_PER_TX,
    MAX_UNENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX
},
    traits::Empty
};

struct PublicAccumulatedDataBuilder {
    note_hashes: BoundedVec<NoteHash, MAX_NOTE_HASHES_PER_TX>,
    nullifiers: BoundedVec<Nullifier, MAX_NULLIFIERS_PER_TX>,
    l2_to_l1_msgs: BoundedVec<Field, MAX_L2_TO_L1_MSGS_PER_TX>,

    note_encrypted_logs_hashes: BoundedVec<LogHash, MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    encrypted_logs_hashes: BoundedVec<LogHash, MAX_ENCRYPTED_LOGS_PER_TX>,
    unencrypted_logs_hashes: BoundedVec<ScopedLogHash, MAX_UNENCRYPTED_LOGS_PER_TX>,

    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,

    public_call_stack: BoundedVec<CallRequest, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,

    gas_used: Gas,
}

impl PublicAccumulatedDataBuilder {
    pub fn finish(self) -> PublicAccumulatedData {
        PublicAccumulatedData {
            note_hashes: self.note_hashes.storage,
            nullifiers: self.nullifiers.storage,
            l2_to_l1_msgs: self.l2_to_l1_msgs.storage,
            note_encrypted_logs_hashes: self.note_encrypted_logs_hashes.storage,
            encrypted_logs_hashes: self.encrypted_logs_hashes.storage,
            unencrypted_logs_hashes: self.unencrypted_logs_hashes.storage,
            public_data_update_requests: self.public_data_update_requests.storage,
            public_call_stack: self.public_call_stack.storage,
            gas_used: self.gas_used
        }
    }
}

impl Empty for PublicAccumulatedDataBuilder {
    fn empty() -> Self {
        PublicAccumulatedDataBuilder {
            note_hashes: BoundedVec::new(),
            nullifiers: BoundedVec::new(),
            l2_to_l1_msgs: BoundedVec::new(),
            note_encrypted_logs_hashes: BoundedVec::new(),
            encrypted_logs_hashes: BoundedVec::new(),
            unencrypted_logs_hashes: BoundedVec::new(),
            public_data_update_requests: BoundedVec::new(),
            public_call_stack: BoundedVec::new(),
            gas_used: Gas::empty(),
        }
    }
}
