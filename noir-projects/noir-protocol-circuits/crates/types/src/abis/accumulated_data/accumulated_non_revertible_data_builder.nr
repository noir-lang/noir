use crate::{
    abis::{
    accumulated_data::{
    private_accumulated_non_revertible_data::PrivateAccumulatedNonRevertibleData,
    public_accumulated_non_revertible_data::PublicAccumulatedNonRevertibleData
},
    call_request::CallRequest, public_data_update_request::PublicDataUpdateRequest,
    side_effect::{SideEffect, SideEffectLinkedToNoteHash}
}
};
use crate::constants::{
    MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX, MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX,
    MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
};

struct AccumulatedNonRevertibleDataBuilder {
    revert_code: u8,

    new_note_hashes: BoundedVec<SideEffect, MAX_NON_REVERTIBLE_NOTE_HASHES_PER_TX>,
    new_nullifiers: BoundedVec<SideEffectLinkedToNoteHash, MAX_NON_REVERTIBLE_NULLIFIERS_PER_TX>,
    public_call_stack: BoundedVec<CallRequest, MAX_NON_REVERTIBLE_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    
    public_data_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_NON_REVERTIBLE_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
}

impl AccumulatedNonRevertibleDataBuilder {
    pub fn to_private(self) -> PrivateAccumulatedNonRevertibleData {
        PrivateAccumulatedNonRevertibleData {
            revert_code: self.revert_code,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            public_call_stack: self.public_call_stack.storage
        }
    }
    pub fn to_public(self) -> PublicAccumulatedNonRevertibleData {
        PublicAccumulatedNonRevertibleData {
            revert_code: self.revert_code,
            new_note_hashes: self.new_note_hashes.storage,
            new_nullifiers: self.new_nullifiers.storage,
            public_call_stack: self.public_call_stack.storage,
            public_data_update_requests: self.public_data_update_requests.storage
        }
    }
}
