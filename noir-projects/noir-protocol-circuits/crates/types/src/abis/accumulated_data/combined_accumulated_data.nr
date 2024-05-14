use crate::{
    hash::compute_tx_logs_hash,
    abis::{
    accumulated_data::public_accumulated_data::PublicAccumulatedData, note_hash::NoteHash,
    nullifier::Nullifier, public_data_update_request::PublicDataUpdateRequest, side_effect::SideEffect,
    gas::Gas
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, COMBINED_ACCUMULATED_DATA_LENGTH
},
    utils::arrays::array_merge, traits::{Empty, Serialize, Deserialize}, utils::reader::Reader
};

struct CombinedAccumulatedData {
    new_note_hashes: [Field; MAX_NEW_NOTE_HASHES_PER_TX],
    new_nullifiers: [Field; MAX_NEW_NULLIFIERS_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hash: Field,
    unencrypted_logs_hash: Field,

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],

    gas_used: Gas,
}

impl CombinedAccumulatedData {
    pub fn combine(non_revertible: PublicAccumulatedData, revertible: PublicAccumulatedData) -> Self {
        // TODO(Miranda): Hash here or elsewhere?
        let encrypted_logs_hash = compute_tx_logs_hash(
            array_merge(
                non_revertible.encrypted_logs_hashes,
                revertible.encrypted_logs_hashes
            )
        );
        let unencrypted_logs_hash = compute_tx_logs_hash(
            array_merge(
                non_revertible.unencrypted_logs_hashes,
                revertible.unencrypted_logs_hashes
            )
        );
        CombinedAccumulatedData {
            new_note_hashes: array_merge(non_revertible.new_note_hashes, revertible.new_note_hashes).map(|n: NoteHash| n.value),
            new_nullifiers: array_merge(non_revertible.new_nullifiers, revertible.new_nullifiers).map(|n: Nullifier| n.value),
            new_l2_to_l1_msgs: array_merge(
                non_revertible.new_l2_to_l1_msgs,
                revertible.new_l2_to_l1_msgs
            ),
            encrypted_logs_hash,
            unencrypted_logs_hash,
            encrypted_log_preimages_length: non_revertible.encrypted_log_preimages_length
                + revertible.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: non_revertible.unencrypted_log_preimages_length
                + revertible.unencrypted_log_preimages_length,
            public_data_update_requests: array_merge(
                non_revertible.public_data_update_requests,
                revertible.public_data_update_requests
            ),
            gas_used: revertible.gas_used + non_revertible.gas_used
        }
    }
}

impl Empty for CombinedAccumulatedData {
    fn empty() -> Self {
        CombinedAccumulatedData {
            new_note_hashes: [0; MAX_NEW_NOTE_HASHES_PER_TX],
            new_nullifiers: [0; MAX_NEW_NULLIFIERS_PER_TX],
            new_l2_to_l1_msgs: [0; MAX_NEW_L2_TO_L1_MSGS_PER_TX],
            encrypted_logs_hash: 0,
            unencrypted_logs_hash: 0,
            encrypted_log_preimages_length: 0,
            unencrypted_log_preimages_length: 0,
            public_data_update_requests: [PublicDataUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
            gas_used: Gas::empty()
        }
    }
}

impl Serialize<COMBINED_ACCUMULATED_DATA_LENGTH> for CombinedAccumulatedData {
  fn serialize(self) -> [Field; COMBINED_ACCUMULATED_DATA_LENGTH] {
    let mut fields: BoundedVec<Field, COMBINED_ACCUMULATED_DATA_LENGTH> = BoundedVec::new();

    fields.extend_from_array(self.new_note_hashes);
    fields.extend_from_array(self.new_nullifiers);
    fields.extend_from_array(self.new_l2_to_l1_msgs);
    fields.push(self.encrypted_logs_hash);
    fields.push(self.unencrypted_logs_hash);
    fields.push(self.encrypted_log_preimages_length);
    fields.push(self.unencrypted_log_preimages_length);

    for i in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
      fields.extend_from_array(self.public_data_update_requests[i].serialize());
    }

    fields.extend_from_array(self.gas_used.serialize());

    assert_eq(fields.len(), COMBINED_ACCUMULATED_DATA_LENGTH);

    fields.storage
  }
}

impl Deserialize<COMBINED_ACCUMULATED_DATA_LENGTH> for CombinedAccumulatedData {
  fn deserialize(fields: [Field; COMBINED_ACCUMULATED_DATA_LENGTH]) -> CombinedAccumulatedData {
    let mut reader = Reader::new(fields);

    let item = CombinedAccumulatedData {
        new_note_hashes: reader.read_array([0; MAX_NEW_NOTE_HASHES_PER_TX]),
        new_nullifiers: reader.read_array([0; MAX_NEW_NULLIFIERS_PER_TX]),
        new_l2_to_l1_msgs: reader.read_array([0; MAX_NEW_L2_TO_L1_MSGS_PER_TX]),
        encrypted_logs_hash: reader.read(),
        unencrypted_logs_hash: reader.read(),
        encrypted_log_preimages_length: reader.read(),
        unencrypted_log_preimages_length: reader.read(),
        public_data_update_requests: reader.read_struct_array(PublicDataUpdateRequest::deserialize, [PublicDataUpdateRequest::empty(); MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX]),
        gas_used: reader.read_struct(Gas::deserialize),
    };
    reader.finish();
    item
  }
}

impl Eq for CombinedAccumulatedData {
  fn eq(self, other: Self) -> bool {
    (self.new_note_hashes == other.new_note_hashes) &
    (self.new_nullifiers == other.new_nullifiers) &
    (self.new_l2_to_l1_msgs == other.new_l2_to_l1_msgs) &
    (self.encrypted_logs_hash == other.encrypted_logs_hash) &
    (self.unencrypted_logs_hash == other.unencrypted_logs_hash) &
    (self.encrypted_log_preimages_length == other.encrypted_log_preimages_length) &
    (self.unencrypted_log_preimages_length == other.unencrypted_log_preimages_length) &
    (self.public_data_update_requests == other.public_data_update_requests) &
    (self.gas_used == other.gas_used)
  }
}

#[test]
fn serialization_of_empty() {
    let item = CombinedAccumulatedData::empty();
    let serialized = item.serialize();
    let deserialized = CombinedAccumulatedData::deserialize(serialized);
    assert(item.eq(deserialized));
}
