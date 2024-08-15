use dep::types::{
    traits::Empty, abis::{validation_requests::ScopedKeyValidationRequestAndGenerator}, scalar::Scalar,
    hash::poseidon2_hash_with_separator
};
use std::embedded_curve_ops::fixed_base_scalar_mul as derive_public_key;

struct KeyValidationHint {
  sk_m: Scalar,
  request_index: u32,
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

impl KeyValidationHint {
    pub fn nada(request_len: u32) -> Self {
        KeyValidationHint { sk_m: Scalar::empty(), request_index: request_len }
    }
}

fn verify_key_validation_request(scoped_request: ScopedKeyValidationRequestAndGenerator, sk_m: Scalar) {
    let contract_address = scoped_request.contract_address;
    let request_and_generator = scoped_request.request;
    let request = request_and_generator.request;
    let sk_app_generator = request_and_generator.sk_app_generator;

    // First we check that derived public key matches master public key from request.
    let pk_m = derive_public_key(sk_m);
    assert_eq(pk_m, request.pk_m, "Failed to derive matching master public key from the secret key.");

    // Then we check that siloing the master secret key with the contract address gives the app secret key.
    let sk_app = poseidon2_hash_with_separator(
        [sk_m.hi, sk_m.lo, contract_address.to_field()],
        sk_app_generator
    );
    assert_eq(sk_app, request.sk_app, "Failed to derive matching app secret key from the secret key.");
}

unconstrained fn get_hint_indexes<let REQUEST_LEN: u32, let NUM_HINTS: u32>(hints: [KeyValidationHint; NUM_HINTS]) -> [u32; REQUEST_LEN] {
    let mut hint_indexes = [NUM_HINTS; REQUEST_LEN];
    for i in 0..NUM_HINTS {
        let hint = hints[i];
        if hint.request_index != REQUEST_LEN {
            hint_indexes[hint.request_index] = i;
        }
    }
    hint_indexes
}

pub fn verify_reset_key_validation_requests<let REQUEST_LEN: u32, let NUM_HINTS: u32>(
    key_validation_requests: [ScopedKeyValidationRequestAndGenerator; REQUEST_LEN],
    hints: [KeyValidationHint; NUM_HINTS],
    unverified_requests: [ScopedKeyValidationRequestAndGenerator; REQUEST_LEN]
) {
    for i in 0..NUM_HINTS {
        let hint = hints[i];
        // Determining here whether a key validation request will be validated based on a hint is not a vulnerability
        // because in the tail circuit we verify that there are no requests remaining to be validated. For this reason
        // the circuits cannot be tricked by not providing a hint (the final check would fail).
        if hint.request_index != key_validation_requests.len() {
            verify_key_validation_request(key_validation_requests[hint.request_index], hint.sk_m);
        }
    }

    let hint_indexes: [u32; REQUEST_LEN] = get_hint_indexes(hints);
    let mut num_propagated = 0;
    for i in 0..REQUEST_LEN {
        let hint_index = hint_indexes[i];
        if hint_index == NUM_HINTS {
            assert_eq(
                unverified_requests[num_propagated], key_validation_requests[i], "mismatch propagated key validation request"
            );
            num_propagated += 1;
        } else {
            assert_eq(hints[hint_index].request_index, i, "wrong verified request index hint");
        }
    }
}

unconstrained pub fn get_unverified_key_validation_requests<let REQUEST_LEN: u32, let NUM_HINTS: u32>(
    key_validation_requests: [ScopedKeyValidationRequestAndGenerator; REQUEST_LEN],
    hints: [KeyValidationHint; NUM_HINTS]
) -> [ScopedKeyValidationRequestAndGenerator; REQUEST_LEN] {
    let mut should_propagate = [true; REQUEST_LEN];
    for i in 0..NUM_HINTS {
        let hint = hints[i];
        if hint.request_index != REQUEST_LEN {
            should_propagate[hint.request_index] = false;
        }
    }

    let mut propagated = BoundedVec::new();
    for i in 0..REQUEST_LEN {
        if should_propagate[i] {
            propagated.push(key_validation_requests[i]);
        }
    }
    propagated.storage
}
