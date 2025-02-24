// Adapted from https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/ecc/groups/affine_element.rs
//!
//! Code is used under the MIT license

use acvm_blackbox_solver::blake3;

use ark_ec::AffineRepr;
use ark_ff::Field;
use ark_ff::{BigInteger, PrimeField};
use ark_grumpkin::{Affine, Fq};

/// Hash a seed buffer into a point
///
/// # ALGORITHM DESCRIPTION
///
///  1. Initialize unsigned integer `attempt_count = 0`
///  2. Copy seed into a buffer whose size is 2 bytes greater than `seed` (initialized to `0`)
///  3. Interpret `attempt_count` as a byte and write into buffer at `[buffer.size() - 2]`
///  4. Compute Blake3 hash of buffer
///  5. Set the end byte of the buffer to `1`
///  6. Compute Blake3 hash of buffer
///  7. Interpret the two hash outputs as the high / low 256 bits of a 512-bit integer (big-endian)
///  8. Derive x-coordinate of point by reducing the 512-bit integer modulo the curve's field modulus (Fq)
///  9. Compute `y^2` from the curve formula `y^2 = x^3 + ax + b` (`a`, `b` are curve params. for BN254, `a = 0`, `b = 3`)
///  10. IF `y^2` IS NOT A QUADRATIC RESIDUE:
///
///  a. increment `attempt_count` by 1 and go to step 2
///  
///  11. IF `y^2` IS A QUADRATIC RESIDUE:
///
///  a. derive y coordinate via `y = sqrt(y)`
///
///  b. Interpret most significant bit of 512-bit integer as a 'parity' bit
///
///  c. If parity bit is set AND `y`'s most significant bit is not set, invert `y`
///
///  d. If parity bit is not set AND `y`'s most significant bit is set, invert `y`
///
///  e. return (x, y)
///
///  N.B. steps c. and e. are because the `sqrt()` algorithm can return 2 values,
///  we need to a way to canonically distinguish between these 2 values and select a "preferred" one
pub(crate) fn hash_to_curve(seed: &[u8], attempt_count: u8) -> Affine {
    let seed_size = seed.len();
    // expand by 2 bytes to cover incremental hash attempts
    let mut target_seed = seed.to_vec();
    target_seed.extend_from_slice(&[0u8; 2]);

    target_seed[seed_size] = attempt_count;
    target_seed[seed_size + 1] = 0;
    let hash_hi = blake3(&target_seed).expect("hash should succeed");
    target_seed[seed_size + 1] = 1;
    let hash_lo = blake3(&target_seed).expect("hash should succeed");

    let mut hash = hash_hi.to_vec();
    hash.extend_from_slice(&hash_lo);

    // Here we reduce the 512 bit number modulo the base field modulus to calculate `x`
    let x = Fq::from_be_bytes_mod_order(&hash);
    let x = Fq::from_base_prime_field(x);

    if let Some(point) = Affine::get_point_from_x_unchecked(x, false) {
        let parity_bit = hash_hi[0] > 127;
        let y_bit_set = point.y().unwrap().into_bigint().get_bit(0);
        if (parity_bit && !y_bit_set) || (!parity_bit && y_bit_set) {
            -point
        } else {
            point
        }
    } else {
        hash_to_curve(seed, attempt_count + 1)
    }
}

#[cfg(test)]
mod test {

    use ark_ec::AffineRepr;
    use ark_ff::{BigInteger, PrimeField};

    use super::hash_to_curve;

    #[test]
    fn smoke_test() {
        let test_cases: [(&[u8], u8, (&str, &str)); 4] = [
            (
                &[],
                0,
                (
                    "24c4cb9c1206ab5470592f237f1698abe684dadf0ab4d7a132c32b2134e2c12e",
                    "0668b8d61a317fb34ccad55c930b3554f1828a0e5530479ecab4defe6bbc0b2e",
                ),
            ),
            (
                &[],
                1,
                (
                    "24c4cb9c1206ab5470592f237f1698abe684dadf0ab4d7a132c32b2134e2c12e",
                    "0668b8d61a317fb34ccad55c930b3554f1828a0e5530479ecab4defe6bbc0b2e",
                ),
            ),
            (
                &[1],
                0,
                (
                    "107f1b633c6113f3222f39f6256f0546b41a4880918c86864b06471afb410454",
                    "050cd3823d0c01590b6a50adcc85d2ee4098668fd28805578aa05a423ea938c6",
                ),
            ),
            (
                &[0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64],
                0,
                (
                    "037c5c229ae495f6e8d1b4bf7723fafb2b198b51e27602feb8a4d1053d685093",
                    "10cf9596c5b2515692d930efa2cf3817607e4796856a79f6af40c949b066969f",
                ),
            ),
        ];

        for (seed, attempt_count, expected_point) in test_cases {
            let point = hash_to_curve(seed, attempt_count);
            assert!(point.is_on_curve());
            assert_eq!(
                hex::encode(point.x().unwrap().into_bigint().to_bytes_be()),
                expected_point.0,
                "Failed on x component with seed {seed:?}, attempt_count {attempt_count}"
            );
            assert_eq!(
                hex::encode(point.y().unwrap().into_bigint().to_bytes_be()),
                expected_point.1,
                "Failed on y component with seed {seed:?}, attempt_count {attempt_count}"
            );
        }
    }
}
