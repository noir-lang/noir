use crate::{
    note_hash_read_request_reset::NoteHashReadRequestHints,
    nullifier_read_request_reset::NullifierReadRequestHints, reset::read_request::reset_read_requests,
    reset::key_validation_hint::{KeyValidationHint, reset_key_validation_requests}
};
use dep::types::{
    abis::{
    note_hash::ScopedNoteHash, nullifier::ScopedNullifier,
    validation_requests::{ValidationRequests, ScopedKeyValidationRequestAndGenerator},
    read_request::ScopedReadRequest
},
    constants::{
    MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_KEY_VALIDATION_REQUESTS_PER_TX,
    GENERATOR_INDEX__NSK_M, MAX_NOTE_HASH_READ_REQUESTS_PER_TX, MAX_NULLIFIER_READ_REQUESTS_PER_TX
},
    scalar::Scalar, hash::poseidon2_hash, traits::is_empty, utils::arrays::filter_array_to_bounded_vec
};

struct PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> {
    validation_requests: ValidationRequests,
    note_hash_read_request_hints: NoteHashReadRequestHints<NH_RR_PENDING, NH_RR_SETTLED>,
    pending_note_hashes: [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX],
    note_hash_tree_root: Field,
    nullifier_read_request_hints: NullifierReadRequestHints<NLL_RR_PENDING, NLL_RR_SETTLED>,
    pending_nullifiers: [ScopedNullifier; MAX_NULLIFIERS_PER_TX],
    nullifier_tree_root: Field,
    key_validation_hints: [KeyValidationHint; KEY_VALIDATION_REQUESTS],
}

impl<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> PrivateValidationRequestProcessor<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> {
    pub fn validate(self) -> ValidationRequests {
        let remaining_note_hash_read_requests = self.validate_note_hash_read_requests();
        let remaining_nullifier_read_requests = self.validate_nullifier_read_requests();
        let remaining_key_validation_requests = self.validate_keys();

        ValidationRequests {
            for_rollup: self.validation_requests.for_rollup,
            nullifier_non_existent_read_requests: self.validation_requests.nullifier_non_existent_read_requests,
            public_data_reads: self.validation_requests.public_data_reads,
            note_hash_read_requests: remaining_note_hash_read_requests.storage,
            nullifier_read_requests: remaining_nullifier_read_requests.storage,
            scoped_key_validation_requests_and_generators: remaining_key_validation_requests.storage
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

    fn validate_keys(self) -> BoundedVec<ScopedKeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_TX> {
        reset_key_validation_requests(
            self.validation_requests.scoped_key_validation_requests_and_generators,
            self.key_validation_hints
        )
    }
}
