// Adapted from https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/ecc/groups/affine_element.rshttps://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/ecc/groups/group.rs
//!
//! Code is used under the MIT license

use std::sync::OnceLock;

use acvm_blackbox_solver::blake3;
use ark_grumpkin::Affine;

use super::hash_to_curve::hash_to_curve;

pub(crate) const DEFAULT_DOMAIN_SEPARATOR: &[u8] = "DEFAULT_DOMAIN_SEPARATOR".as_bytes();
const NUM_DEFAULT_GENERATORS: usize = 8;

fn default_generators() -> &'static [Affine; NUM_DEFAULT_GENERATORS] {
    static INSTANCE: OnceLock<[Affine; NUM_DEFAULT_GENERATORS]> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        _derive_generators(DEFAULT_DOMAIN_SEPARATOR, NUM_DEFAULT_GENERATORS as u32, 0)
            .try_into()
            .expect("Should generate `NUM_DEFAULT_GENERATORS`")
    })
}

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
pub fn derive_generators(
    domain_separator_bytes: &[u8],
    num_generators: u32,
    starting_index: u32,
) -> Vec<Affine> {
    // We cache a small number of the default generators so we can reuse them without needing to repeatedly recalculate them.
    if domain_separator_bytes == DEFAULT_DOMAIN_SEPARATOR
        && starting_index + num_generators <= NUM_DEFAULT_GENERATORS as u32
    {
        let start_index = starting_index as usize;
        let end_index = (starting_index + num_generators) as usize;
        default_generators()[start_index..end_index].to_vec()
    } else {
        _derive_generators(domain_separator_bytes, num_generators, starting_index)
    }
}

fn _derive_generators(
    domain_separator_bytes: &[u8],
    num_generators: u32,
    starting_index: u32,
) -> Vec<Affine> {
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
        let generator = hash_to_curve(&generator_preimage, 0);
        res.push(generator);
    }
    res
}

#[cfg(test)]
mod test {

    use ark_ec::AffineRepr;
    use ark_ff::{BigInteger, PrimeField};

    use super::*;

    #[test]
    fn test_derive_generators() {
        let res = derive_generators("test domain".as_bytes(), 128, 0);

        let is_unique = |y: Affine, j: usize| -> bool {
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

    #[test]
    fn derive_length_generator() {
        let domain_separator = "pedersen_hash_length";
        let length_generator = derive_generators(domain_separator.as_bytes(), 1, 0)[0];

        let expected_generator = (
            "2df8b940e5890e4e1377e05373fae69a1d754f6935e6a780b666947431f2cdcd",
            "2ecd88d15967bc53b885912e0d16866154acb6aac2d3f85e27ca7eefb2c19083",
        );
        assert_eq!(
            hex::encode(length_generator.x().unwrap().into_bigint().to_bytes_be()),
            expected_generator.0,
            "Failed on x component"
        );
        assert_eq!(
            hex::encode(length_generator.y().unwrap().into_bigint().to_bytes_be()),
            expected_generator.1,
            "Failed on y component"
        );
    }

    #[test]
    fn derives_default_generators() {
        const DEFAULT_GENERATORS: &[[&str; 2]] = &[
            [
                "083e7911d835097629f0067531fc15cafd79a89beecb39903f69572c636f4a5a",
                "1a7f5efaad7f315c25a918f30cc8d7333fccab7ad7c90f14de81bcc528f9935d",
            ],
            [
                "054aa86a73cb8a34525e5bbed6e43ba1198e860f5f3950268f71df4591bde402",
                "209dcfbf2cfb57f9f6046f44d71ac6faf87254afc7407c04eb621a6287cac126",
            ],
            [
                "1c44f2a5207c81c28a8321a5815ce8b1311024bbed131819bbdaf5a2ada84748",
                "03aaee36e6422a1d0191632ac6599ae9eba5ac2c17a8c920aa3caf8b89c5f8a8",
            ],
            [
                "26d8b1160c6821a30c65f6cb47124afe01c29f4338f44d4a12c9fccf22fb6fb2",
                "05c70c3b9c0d25a4c100e3a27bf3cc375f8af8cdd9498ec4089a823d7464caff",
            ],
            [
                "20ed9c6a1d27271c4498bfce0578d59db1adbeaa8734f7facc097b9b994fcf6e",
                "29cd7d370938b358c62c4a00f73a0d10aba7e5aaa04704a0713f891ebeb92371",
            ],
            [
                "0224a8abc6c8b8d50373d64cd2a1ab1567bf372b3b1f7b861d7f01257052d383",
                "2358629b90eafb299d6650a311e79914b0215eb0a790810b26da5a826726d711",
            ],
            [
                "0f106f6d46bc904a5290542490b2f238775ff3c445b2f8f704c466655f460a2a",
                "29ab84d472f1d33f42fe09c47b8f7710f01920d6155250126731e486877bcf27",
            ],
            [
                "0298f2e42249f0519c8a8abd91567ebe016e480f219b8c19461d6a595cc33696",
                "035bec4b8520a4ece27bd5aafabee3dfe1390d7439c419a8c55aceb207aac83b",
            ],
        ];

        let generated_generators =
            derive_generators(DEFAULT_DOMAIN_SEPARATOR, DEFAULT_GENERATORS.len() as u32, 0);
        for (i, (generator, expected_generator)) in
            generated_generators.iter().zip(DEFAULT_GENERATORS).enumerate()
        {
            assert_eq!(
                hex::encode(generator.x().unwrap().into_bigint().to_bytes_be()),
                expected_generator[0],
                "Failed on x component of generator {i}"
            );
            assert_eq!(
                hex::encode(generator.y().unwrap().into_bigint().to_bytes_be()),
                expected_generator[1],
                "Failed on y component of generator {i}"
            );
        }
    }
}
