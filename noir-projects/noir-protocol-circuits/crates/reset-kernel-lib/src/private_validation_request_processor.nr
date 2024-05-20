use crate::{
    note_hash_read_request_reset::NoteHashReadRequestHints,
    nullifier_read_request_reset::NullifierReadRequestHints, reset::read_request::reset_read_requests,
    reset::nullifier_key::{NullifierKeyHint, reset_nullifier_keys}
};
use dep::types::{
    abis::{
    note_hash::ScopedNoteHash, nullifier::ScopedNullifier, validation_requests::ValidationRequests,
    read_request::ScopedReadRequest,
    nullifier_key_validation_request::ScopedNullifierKeyValidationRequest
},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
    GENERATOR_INDEX__NSK_M, MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX
},
    grumpkin_private_key::GrumpkinPrivateKey, hash::poseidon2_hash, traits::is_empty,
    utils::arrays::filter_array_to_bounded_vec
};

struct PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, NLL_KEYS> {
    validation_requests: ValidationRequests,
    note_hash_read_request_hints: NoteHashReadRequestHints<NH_RR_PENDING, NH_RR_SETTLED>,
    pending_note_hashes: [ScopedNoteHash; MAX_NEW_NOTE_HASHES_PER_TX],
    note_hash_tree_root: Field,
    nullifier_read_request_hints: NullifierReadRequestHints<NLL_RR_PENDING, NLL_RR_SETTLED>,
    pending_nullifiers: [ScopedNullifier; MAX_NEW_NULLIFIERS_PER_TX],
    nullifier_tree_root: Field,
    master_nullifier_secret_keys: [NullifierKeyHint; NLL_KEYS],
}

impl<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, NLL_KEYS> PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, NLL_KEYS> {
    pub fn validate(self) -> ValidationRequests {
        let remaining_note_hash_read_requests = self.validate_note_hash_read_requests();
        let remaining_nullifier_read_requests = self.validate_nullifier_read_requests();
        let remaining_nullifier_key_validation_requests = self.validate_nullifier_keys();

        ValidationRequests {
            for_rollup: self.validation_requests.for_rollup,
            nullifier_non_existent_read_requests: self.validation_requests.nullifier_non_existent_read_requests,
            public_data_reads: self.validation_requests.public_data_reads,
            note_hash_read_requests: remaining_note_hash_read_requests.storage,
            nullifier_read_requests: remaining_nullifier_read_requests.storage,
            nullifier_key_validation_requests: remaining_nullifier_key_validation_requests.storage
        }
    }

    fn validate_note_hash_read_requests(self) -> BoundedVec<ScopedReadRequest, MAX_NOTE_HASH_READ_REQUESTS_PER_TX> {
        reset_read_requests(
            self.validation_requests.note_hash_read_requests,
            self.pending_note_hashes,
            self.note_hash_read_request_hints.read_request_statuses,
            self.note_hash_read_request_hints.pending_read_hints,
            self.note_hash_read_request_hints.settled_read_hints,
            self.note_hash_tree_root
        )
    }

    fn validate_nullifier_read_requests(self) -> BoundedVec<ScopedReadRequest, MAX_NULLIFIER_READ_REQUESTS_PER_TX> {
        reset_read_requests(
            self.validation_requests.nullifier_read_requests,
            self.pending_nullifiers,
            self.nullifier_read_request_hints.read_request_statuses,
            self.nullifier_read_request_hints.pending_read_hints,
            self.nullifier_read_request_hints.settled_read_hints,
            self.nullifier_tree_root
        )
    }

    fn validate_nullifier_keys(self) -> BoundedVec<ScopedNullifierKeyValidationRequest, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX> {
        reset_nullifier_keys(
            self.validation_requests.nullifier_key_validation_requests,
            self.master_nullifier_secret_keys
        )
    }
}
