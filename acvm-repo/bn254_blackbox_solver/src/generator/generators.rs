// Taken from https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/ecc/groups/affine_element.rshttps://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/ecc/groups/group.rs

use ark_ec::short_weierstrass::{Affine, SWCurveConfig};

use acvm_blackbox_solver::blake3;

use super::hash_to_curve::hash_to_curve;

/// Derives generator points via [hash-to-curve][hash_to_curve].
///
/// # ALGORITHM DESCRIPTION
///
/// 1. Each generator has an associated "generator index" described by its location in the vector
/// 2. a 64-byte preimage buffer is generated with the following structure:
///     - bytes 0-31: BLAKE3 hash of domain_separator
///     - bytes 32-63: generator index in big-endian form
/// 3. The [hash-to-curve algorithm][hash_to_curve] is used to hash the above into a curve point.
///
/// NOTE: The domain separator is included to ensure that it is possible to derive independent sets of
/// index-addressable generators.
///
/// [hash_to_curve]: super::hash_to_curve::hash_to_curve
pub(crate) fn derive_generators<E: SWCurveConfig>(
    domain_separator_bytes: &[u8],
    num_generators: u32,
    starting_index: u32,
) -> Vec<Affine<E>> {
    let mut generator_preimage = [0u8; 64];
    let domain_hash = blake3(domain_separator_bytes).expect("hash should succeed");
    //1st 32 bytes are blake3 domain_hash
    generator_preimage[..32].copy_from_slice(&domain_hash);

    // Convert generator index in big-endian form
    let mut res = Vec::with_capacity(num_generators as usize);
    for i in starting_index..(starting_index + num_generators) {
        let generator_be_bytes: [u8; 4] = i.to_be_bytes();
        generator_preimage[32] = generator_be_bytes[0];
        generator_preimage[33] = generator_be_bytes[1];
        generator_preimage[34] = generator_be_bytes[2];
        generator_preimage[35] = generator_be_bytes[3];
        res.push(hash_to_curve(&generator_preimage, 0));
    }
    res
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_derive_generators() {
        let res =
            derive_generators::<grumpkin::GrumpkinParameters>("test domain".as_bytes(), 128, 0);

        let is_unique = |y: Affine<grumpkin::GrumpkinParameters>, j: usize| -> bool {
            for (i, res) in res.iter().enumerate() {
                if i != j && *res == y {
                    return false;
                }
            }
            true
        };

        for (i, res) in res.iter().enumerate() {
            assert!(is_unique(*res, i));
            assert!(res.is_on_curve());
        }
    }
}
