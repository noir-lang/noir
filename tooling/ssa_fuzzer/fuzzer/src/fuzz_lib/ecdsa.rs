use rand::{Rng, RngCore, SeedableRng, rngs::StdRng};

use sha2::{Digest, Sha256};

/// Creates a deterministic RNG seeded from the message hash.
/// This ensures the same message always produces the same key/signature.
fn rng_from_msg(msg: &[u8]) -> StdRng {
    let hash = Sha256::digest(msg);
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&hash);
    StdRng::from_seed(seed)
}

#[derive(Debug)]
pub(crate) struct SignatureSsaPrepared {
    pub(crate) hash: Vec<u8>,
    pub(crate) public_key_x: Vec<u8>,
    pub(crate) public_key_y: Vec<u8>,
    pub(crate) signature: Vec<u8>,
}

/// Signs message with secp256k1 curve with random key and returns signature in prepared format
fn generate_ecdsa_signature_secp256k1_internal(msg: &[u8]) -> SignatureSsaPrepared {
    use k256::ecdsa::{Signature, SigningKey, VerifyingKey, signature::Signer};
    use k256::elliptic_curve::scalar::IsHigh;
    let mut rng = rng_from_msg(msg);
    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);
    let signing_key = SigningKey::from_slice(&key_bytes).expect("valid k256 signing key");
    let signature: Signature = signing_key.sign(msg);
    let verifying_key = VerifyingKey::from(&signing_key); // == public key
    let public_key_bytes = verifying_key.to_sec1_point(/*compress = */ false).to_bytes();
    let signature_bytes = if signature.s().is_high().into() {
        signature.normalize_s().to_bytes()
    } else {
        signature.to_bytes()
    };
    let hash = Sha256::digest(msg);
    SignatureSsaPrepared {
        hash: hash.to_vec(),
        public_key_x: public_key_bytes[1..33].to_vec(),
        public_key_y: public_key_bytes[33..65].to_vec(),
        signature: signature_bytes.to_vec(),
    }
}

/// Signs message with secp256r1 curve with random key and returns signature in prepared format
fn generate_ecdsa_signature_secp256r1_internal(msg: &[u8]) -> SignatureSsaPrepared {
    use p256::ecdsa::{Signature, SigningKey, VerifyingKey, signature::Signer};
    use p256::elliptic_curve::scalar::IsHigh;
    let mut rng = rng_from_msg(msg);
    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);
    let signing_key = SigningKey::from_slice(&key_bytes).expect("valid p256 signing key");
    let signature: Signature = signing_key.sign(msg);
    let verifying_key = VerifyingKey::from(&signing_key); // == public key
    let public_key_bytes = verifying_key.to_sec1_point(/*compress = */ false).to_bytes();
    let signature_bytes = if signature.s().is_high().into() {
        signature.normalize_s().to_bytes()
    } else {
        signature.to_bytes()
    };
    let hash = Sha256::digest(msg);
    SignatureSsaPrepared {
        hash: hash.to_vec(),
        public_key_x: public_key_bytes[1..33].to_vec(),
        public_key_y: public_key_bytes[33..65].to_vec(),
        signature: signature_bytes.to_vec(),
    }
}

pub(crate) enum Curve {
    Secp256k1,
    Secp256r1,
}

/// Signs message with secp256k1 or secp256r1 curve with random key and
/// returns signature in prepared format
/// Corrupts the hash, public key x, public key y, or signature if the corresponding flag is true
pub(crate) fn generate_ecdsa_signature_and_corrupt_it(
    msg: &[u8],
    target_curve: Curve,
    corrupt_hash: bool,
    corrupt_pubkey_x: bool,
    corrupt_pubkey_y: bool,
    corrupt_signature: bool,
) -> SignatureSsaPrepared {
    let mut rng = rng_from_msg(msg);
    let mut prepared_signature = match target_curve {
        Curve::Secp256k1 => generate_ecdsa_signature_secp256k1_internal(msg),
        Curve::Secp256r1 => generate_ecdsa_signature_secp256r1_internal(msg),
    };
    if corrupt_hash {
        let new_size = rng.random_range(u8::MIN..=u8::MAX);
        let mut new_bytes = vec![0; new_size as usize];
        rng.fill_bytes(&mut new_bytes);
        prepared_signature.hash = new_bytes;
    }
    if corrupt_pubkey_x {
        let new_size = rng.random_range(u8::MIN..=u8::MAX);
        let mut new_bytes = vec![0; new_size as usize];
        rng.fill_bytes(&mut new_bytes);
        prepared_signature.public_key_x = new_bytes;
    }
    if corrupt_pubkey_y {
        let new_size = rng.random_range(u8::MIN..=u8::MAX);
        let mut new_bytes = vec![0; new_size as usize];
        rng.fill_bytes(&mut new_bytes);
        prepared_signature.public_key_y = new_bytes;
    }
    if corrupt_signature {
        let new_size = rng.random_range(u8::MIN..=u8::MAX);
        let mut new_bytes = vec![0; new_size as usize];
        rng.fill_bytes(&mut new_bytes);
        prepared_signature.signature = new_bytes;
    }
    prepared_signature
}
