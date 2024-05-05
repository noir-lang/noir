// Taken from https://github.com/laudiacay/barustenberg/blob/df6bc6f095fe7f288bf6a12e7317fd8eb33d68ae/barustenberg/src/ecc/groups/affine_element.rs

use acvm_blackbox_solver::blake3;

use ark_ec::short_weierstrass::{Affine, SWCurveConfig};
use ark_ff::{Field, PrimeField};

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
///  9. Compute `y^2`` from the curve formula `y^2 = x^3 + ax + b` (`a``, `b`` are curve params. for BN254, `a = 0``, `b = 3``)
///  10. IF `y^2`` IS NOT A QUADRATIC RESIDUE
///      
///     a. increment `attempt_count` by 1 and go to step 2
///  
/// 11. IF `y^2`` IS A QUADRATIC RESIDUE
///      
///     a. derive y coordinate via `y = sqrt(y)``
///      
///     b. Interpret most significant bit of 512-bit integer as a 'parity' bit
///     
///     In Barretenberg:
///          11c. If parity bit is set AND `y`'s most significant bit is not set, invert `y`
///          11d. If parity bit is not set AND `y`'s most significant bit is set, invert `y`
///      In Noir we use arkworks https://github.com/arkworks-rs/algebra/blob/master/ec/src/models/short_weierstrass/affine.rs#L110:
///          11c. If parity bit is set AND `y < -y` lexographically, invert `y`
///          11d. If parity bit is not set AND `y >= -y` lexographically, invert `y`
///      N.B. last 2 steps are because the `sqrt()` algorithm can return 2 values,
///           we need to a way to canonically distinguish between these 2 values and select a "preferred" one
///      11e. return (x, y)
///
pub(crate) fn hash_to_curve<E: SWCurveConfig>(seed: &[u8], attempt_count: u8) -> Affine<E> {
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
    let x = <E::BaseField as Field>::BasePrimeField::from_be_bytes_mod_order(&hash);
    let x = E::BaseField::from_base_prime_field(x);

    let parity_bit = hash_hi[0] > 127;
    Affine::get_point_from_x_unchecked(x, parity_bit)
        .unwrap_or_else(|| hash_to_curve(seed, attempt_count + 1))
}

#[cfg(test)]
mod test {
    use ark_ec::AffineRepr;

    use super::hash_to_curve;

    #[test]
    fn smoke_test() {
        // NOTE: test cases are generated from the code above. These need to be checked against barretenberg for consistency!
        let test_cases: [(&[u8], u8, (&str, &str)); 3] = [
            (
                &[],
                0,
                (
                    "16630969835852596693293552682274346428810960863289591405364736021328503685422",
                    "2898904879751428755315857271136646918777181217826622916610882064326950914862",
                ),
            ),
            (
                &[],
                1,
                (
                    "16630969835852596693293552682274346428810960863289591405364736021328503685422",
                    "2898904879751428755315857271136646918777181217826622916610882064326950914862",
                ),
            ),
            (
                &[42],
                0,
                (
                    "10062871819776274344541726704728265607389658727752533003234345551987599533646",
                    "4336697243017116919088392067104139397869626428691238548002362745837171062453",
                ),
            ),
        ];

        for (seed, attempt_count, expected_point) in test_cases {
            let point = hash_to_curve::<grumpkin::GrumpkinParameters>(seed, attempt_count);
            assert!(point.is_on_curve());
            assert_eq!(point.x().unwrap().to_string(), expected_point.0);
            assert_eq!(point.y().unwrap().to_string(), expected_point.1);
        }
    }
}
