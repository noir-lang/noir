use crate::{nullifier_read_request_reset::NullifierReadRequestHints, read_request_reset::reset_read_requests};
use dep::types::{
    abis::{side_effect::{SideEffect, SideEffectLinkedToNoteHash}, validation_requests::ValidationRequests},
    constants::{
    MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
},
    grumpkin_private_key::GrumpkinPrivateKey, keys::compute_siloed_nullifier_secret_key,
    traits::is_empty
};

struct PrivateValidationRequestProcessor {
    validation_requests: ValidationRequests,
    note_hash_read_request_hints: [u64; MAX_NOTE_HASH_READ_REQUESTS_PER_TX],
    pending_note_hashes: [SideEffect; MAX_NEW_NOTE_HASHES_PER_TX],
    nullifier_read_request_hints: NullifierReadRequestHints,
    pending_nullifiers: [SideEffectLinkedToNoteHash; MAX_NEW_NULLIFIERS_PER_TX],
    master_nullifier_secret_keys: [GrumpkinPrivateKey; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
    nullifier_tree_root: Field
}

impl PrivateValidationRequestProcessor {
    pub fn validate(self) {
        self.validate_note_hash_read_requests();
        self.validate_nullifier_read_requests();
        self.validate_nullifier_keys();
    }

    fn validate_note_hash_read_requests(self) {
        let read_requests = self.validation_requests.note_hash_read_requests;
        for i in 0..read_requests.len() {
            let read_request = read_requests[i];
            if (read_request.value != 0) {
                let index = self.note_hash_read_request_hints[i];
                let note_hash = self.pending_note_hashes[index];
                assert_eq(read_request.value, note_hash.value, "Hinted note hash does not match read request");
                assert(
                    read_request.counter > note_hash.counter, "Read request counter must be greater than note hash counter"
                );
            }
        }
    }

    fn validate_nullifier_read_requests(self) {
        let remaining_requests = reset_read_requests(
            self.validation_requests.nullifier_read_requests,
            self.pending_nullifiers,
            self.nullifier_read_request_hints.read_request_statuses,
            self.nullifier_read_request_hints.pending_read_hints,
            self.nullifier_read_request_hints.settled_read_hints,
            self.nullifier_tree_root
        );
        // When we have a separate reset circuit, we can allow unverified requests and process them later after the 
        // corresponding values are added to public inputs in nested executions.
        // But right now, all the request must be cleared in one go.
        assert(remaining_requests.len() == 0, "All nullifier read requests must be verified");
    }

    fn validate_nullifier_keys(self) {
        let requests = self.validation_requests.nullifier_key_validation_requests;
        for i in 0..MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX {
            let request = requests[i];
            if !is_empty(request) {
                let master_secret_key = self.master_nullifier_secret_keys[i];
                let computed_public_key = master_secret_key.derive_public_key();
                assert(
                    computed_public_key.eq(request.public_key), "Cannot derive nullifier public key from the master key."
                );

                let computed_secret_key = compute_siloed_nullifier_secret_key(master_secret_key, request.contract_address);
                assert(
                    computed_secret_key.eq(request.secret_key), "Cannot derive siloed secret key from the master key."
                );
            }
        }
    }
}
