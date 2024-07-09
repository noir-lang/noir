use dep::types::{
    traits::{Empty, is_empty}, abis::{validation_requests::ScopedKeyValidationRequestAndGenerator},
    constants::MAX_KEY_VALIDATION_REQUESTS_PER_TX, scalar::Scalar, hash::poseidon2_hash,
    utils::arrays::filter_array_to_bounded_vec
};

struct KeyValidationHint {
  sk_m: Scalar,
  request_index: u64,
}

impl Empty for KeyValidationHint {
    fn empty() -> Self {
        KeyValidationHint {
            sk_m: Scalar::empty(),
            request_index: 0,
        }
    }
}

impl Eq for KeyValidationHint {
    fn eq(self, other: Self) -> bool {
        self.sk_m.eq(other.sk_m) & self.request_index.eq(other.request_index)
    }

}

pub fn reset_key_validation_requests<N>(
    key_validation_requests: [ScopedKeyValidationRequestAndGenerator; MAX_KEY_VALIDATION_REQUESTS_PER_TX],
    key_validation_hints: [KeyValidationHint; N]
) -> BoundedVec<ScopedKeyValidationRequestAndGenerator, MAX_KEY_VALIDATION_REQUESTS_PER_TX> {
    let mut should_propagate = key_validation_requests.map(|req| !is_empty(req));
    for i in 0..N {
        let hint = key_validation_hints[i];
        // Determining here whether a key validation request will be validated based on a hint is not a vulnerability
        // because in the reset circuit we verify that there are no requests remaining to be validated. For this reason
        // the circuits cannot be tricked by not providing a hint (the final check would fail).
        if !is_empty(hint) {
            let scoped_request = key_validation_requests[hint.request_index];
            let contract_address = scoped_request.contract_address;
            let request_and_generator = scoped_request.request;
            let request = request_and_generator.request;
            let sk_m = hint.sk_m;
            let sk_app_generator = request_and_generator.sk_app_generator;

            // First we check that derived public key matches master public key from request
            let pk_m = sk_m.derive_public_key();
            assert(
                pk_m.eq(request.pk_m), "Failed to derive matching master public key from the secret key."
            );

            // Then we check that siloing the master secret key with the contract address gives the app secret key

            let sk_app = poseidon2_hash([sk_m.hi, sk_m.lo, contract_address.to_field(), sk_app_generator]);
            assert(
                sk_app.eq(request.sk_app), "Failed to derive matching app secret key from the secret key."
            );

            should_propagate[hint.request_index] = false;
        }
    }

    filter_array_to_bounded_vec(key_validation_requests, should_propagate)
}
