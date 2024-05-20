use crate::{nullifier_read_request_reset::NullifierReadRequestHints};
use dep::types::{
    traits::{Empty, is_empty},
    abis::{nullifier_key_validation_request::ScopedNullifierKeyValidationRequest},
    constants::{MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GENERATOR_INDEX__NSK_M},
    grumpkin_private_key::GrumpkinPrivateKey, hash::poseidon2_hash,
    utils::arrays::filter_array_to_bounded_vec
};

struct NullifierKeyHint {
  private_key: GrumpkinPrivateKey,
  request_index: u64,
}

impl Empty for NullifierKeyHint {
    fn empty() -> Self {
        NullifierKeyHint {
            private_key: GrumpkinPrivateKey::empty(),
            request_index: 0,
        }
    }
}

impl Eq for NullifierKeyHint {
    fn eq(self, other: Self) -> bool {
        self.private_key.eq(other.private_key) & self.request_index.eq(other.request_index)
    }

}

pub fn reset_nullifier_keys<N>(
    nullifier_key_validation_requests: [ScopedNullifierKeyValidationRequest; MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX],
    key_hints: [NullifierKeyHint; N]
) -> BoundedVec<ScopedNullifierKeyValidationRequest, MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX> {
    let mut should_propagate = nullifier_key_validation_requests.map(|req| !is_empty(req));
    for i in 0..N {
        let hint = key_hints[i];
        if !is_empty(hint) {
            let scoped_request = nullifier_key_validation_requests[hint.request_index];
            let contract_address = scoped_request.contract_address;
            let request = scoped_request.request;
            let master_nullifier_secret_key = hint.private_key;

            // First we check that derived public key matches master nullifier public key from request
            let master_nullifier_public_key = master_nullifier_secret_key.derive_public_key();
            assert(
                master_nullifier_public_key.eq(request.master_nullifier_public_key), "Failed to derive matching master nullifier public key from the secret key."
            );

            // Then we check that siloing the master secret key with the contract address gives the app nullifier secret key

            let app_nullifier_secret_key = poseidon2_hash(
                [
                master_nullifier_secret_key.high, master_nullifier_secret_key.low, contract_address.to_field(), GENERATOR_INDEX__NSK_M
            ]
            );
            assert(
                app_nullifier_secret_key.eq(request.app_nullifier_secret_key), "Failed to derive matching app nullifier secret key from the secret key."
            );

            should_propagate[hint.request_index] = false;
        }
    }

    filter_array_to_bounded_vec(nullifier_key_validation_requests, should_propagate)
}
