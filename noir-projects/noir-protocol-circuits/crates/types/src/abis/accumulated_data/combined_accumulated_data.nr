use crate::{
    abis::{
    accumulated_data::public_accumulated_data::PublicAccumulatedData,
    public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}, gas::Gas
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
},
    utils::arrays::array_merge, traits::Empty
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
        CombinedAccumulatedData {
            new_note_hashes: array_merge(non_revertible.new_note_hashes, revertible.new_note_hashes).map(|n: SideEffect| n.value),
            new_nullifiers: array_merge(non_revertible.new_nullifiers, revertible.new_nullifiers).map(|n: SideEffectLinkedToNoteHash| n.value),
            new_l2_to_l1_msgs: revertible.new_l2_to_l1_msgs,
            encrypted_logs_hash: revertible.encrypted_logs_hash,
            unencrypted_logs_hash: revertible.unencrypted_logs_hash,
            encrypted_log_preimages_length: revertible.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: revertible.unencrypted_log_preimages_length,
            public_data_update_requests: array_merge(
                non_revertible.public_data_update_requests,
                revertible.public_data_update_requests
            ),
            gas_used: revertible.gas_used.add(non_revertible.gas_used)
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
