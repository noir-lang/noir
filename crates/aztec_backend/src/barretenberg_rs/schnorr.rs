use noir_field::FieldElement;
use std::convert::TryInto;

use super::Barretenberg;
use super::BARRETENBERG;

impl Barretenberg {
    pub fn construct_signature(&mut self, message: &[u8], private_key: [u8; 32]) -> [u8; 64] {
        let (s, e) = barretenberg_wrapper::schnorr::construct_signature(message, private_key);
        let sig_bytes: [u8; 64] = [s, e].concat().try_into().unwrap();
        sig_bytes
    }

    pub fn construct_public_key(&mut self, private_key: [u8; 32]) -> [u8; 64] {
        let result_bytes = barretenberg_wrapper::schnorr::construct_public_key(&private_key);
        result_bytes
    }

    pub fn verify_signature(
        &mut self,
        pub_key: [u8; 64],
        sig: [u8; 64],
        message: &[u8],
    ) -> FieldElement {
        let _m = BARRETENBERG.lock().unwrap();
        let r: bool = barretenberg_wrapper::schnorr::verify_signature(
            pub_key,
            sig[0..32].try_into().unwrap(),
            sig[32..64].try_into().unwrap(),
            message,
        );
        match r {
            false => FieldElement::zero(),
            true => FieldElement::one(),
            _=> unreachable!("verify signature should return a boolean to indicate whether the signature + parameters were valid")
        }

        // Note, currently for Barretenberg plonk, if the signature fails
        // then the whole circuit fails.
    }
}

#[test]
fn basic_interop() {
    let mut barretenberg = Barretenberg::new();

    // First case should pass, standard procedure for Schnorr
    let private_key = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let public_key = barretenberg.construct_public_key(private_key);
    let signature = barretenberg.construct_signature(&message, private_key);
    let result = barretenberg.verify_signature(public_key, signature, &message);
    assert_eq!(result, FieldElement::one());

    // Should fail, since the messages are different
    let private_key = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let public_key = barretenberg.construct_public_key(private_key);
    let signature = barretenberg.construct_signature(&message, private_key);
    let result = barretenberg.verify_signature(public_key, signature, &[0, 2]);
    assert_eq!(result, FieldElement::zero());

    // Should fail, since the signature is not valid
    let private_key = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let signature = [1; 64];

    let public_key = barretenberg.construct_public_key(private_key);
    let result = barretenberg.verify_signature(public_key, signature, &message);
    assert_eq!(result, FieldElement::zero());

    // Should fail, since the public key does not match
    let private_key_a = [1; 32];
    let private_key_b = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let public_key_b = barretenberg.construct_public_key(private_key_b);
    let signature_a = barretenberg.construct_signature(&message, private_key_a);
    let result = barretenberg.verify_signature(public_key_b, signature_a, &message);
    assert_eq!(result, FieldElement::zero());

    // Test the first case again, to check if memory is being freed and overwritten properly
    let private_key = [2; 32];
    let message = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let public_key = barretenberg.construct_public_key(private_key);
    let signature = barretenberg.construct_signature(&message, private_key);
    let result = barretenberg.verify_signature(public_key, signature, &message);
    assert_eq!(result, FieldElement::one());
}
