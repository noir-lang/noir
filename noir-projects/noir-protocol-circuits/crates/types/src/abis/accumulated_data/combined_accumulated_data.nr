use crate::{
    abis::{
    accumulated_data::public_accumulated_data::PublicAccumulatedData,
    public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
},
    utils::arrays::array_merge
};

struct CombinedAccumulatedData {
    new_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX],
    new_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],
    new_l2_to_l1_msgs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX],

    encrypted_logs_hash: Field,
    unencrypted_logs_hash: Field,

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    encrypted_log_preimages_length: Field,
    unencrypted_log_preimages_length: Field,

    public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
}

impl CombinedAccumulatedData {
    pub fn recombine(non_revertible: PublicAccumulatedData, revertible: PublicAccumulatedData) -> Self {
        CombinedAccumulatedData {
            new_note_hashes: array_merge(non_revertible.new_note_hashes, revertible.new_note_hashes),
            new_nullifiers: array_merge(non_revertible.new_nullifiers, revertible.new_nullifiers),
            new_l2_to_l1_msgs: revertible.new_l2_to_l1_msgs,
            encrypted_logs_hash: revertible.encrypted_logs_hash,
            unencrypted_logs_hash: revertible.unencrypted_logs_hash,
            encrypted_log_preimages_length: revertible.encrypted_log_preimages_length,
            unencrypted_log_preimages_length: revertible.unencrypted_log_preimages_length,
            public_data_update_requests: array_merge(
                non_revertible.public_data_update_requests,
                revertible.public_data_update_requests
            )
        }
    }
}
