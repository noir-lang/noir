// ONLY IMPORT ONE OF THESE CONFIGS! The big `config` takes 11 mins to compile.

// SMALL CONFIG *********************************************************************************

// mod smaller_config;

// use crate::smaller_config::{
//     BigNum, BLS12_381_Fr_Params, F, FIELDS_PER_BLOB, LOG_FIELDS_PER_BLOB, NOIR_FIELDS_PER_BLOB,
//     FIELDS_CARRYING_AN_EXTRA_BIT_PER_BLOB, D, D_INV, ROOTS, NEGATIVE_ROOTS
// };

//*********************************************************************************

// BIG CONFIG *********************************************************************

mod config;
mod negative_roots;

use crate::{
    config::{
    BigNum, BLS12_381_Fr_Params, F, FIELDS_PER_BLOB, LOG_FIELDS_PER_BLOB, NOIR_FIELDS_PER_BLOB,
    FIELDS_CARRYING_AN_EXTRA_BIT_PER_BLOB, D, D_INV, ROOTS
},
    negative_roots::NEGATIVE_ROOTS
};

//*********************************************************************************

use std::hash::poseidon2;

global LIMB_MAX = 2.pow_32(120);

unconstrained fn __batch_invert_impl<let N: u32>(mut x: [F; N]) -> [F; N] {
    let mut accumulator: F = BigNum::one();

    let mut temporaries: [F] = &[];
    for i in 0..x.len() {
        temporaries = temporaries.push_back(accumulator);
        if (x[i].__is_zero() == false) {
            accumulator = accumulator.__mul(x[i]);
        }
    }

    accumulator = accumulator.__invmod();
    let mut T0: F = BigNum::new();
    for i in 0..x.len() {
        let idx = x.len() - 1 - i;
        if (x[idx].__is_zero() == false) {
            T0 = accumulator.__mul(temporaries[idx]);
            accumulator = accumulator.__mul(x[idx]);
            x[idx] = T0;
        }
    }
    x
}

// Not used because it resulted in "stack too deep", so it's inlined instead.
unconstrained fn __compute_fracs(z: F, ys: [F; FIELDS_PER_BLOB]) -> [F; FIELDS_PER_BLOB] {
    let mut denoms: [F; FIELDS_PER_BLOB] = [BigNum::new(); FIELDS_PER_BLOB];
    for i in 0..FIELDS_PER_BLOB {
        denoms[i] = z.__add(NEGATIVE_ROOTS[i]); // (z - ω^i)
    }
    let inv_denoms = __batch_invert_impl(denoms); // 1 / (z - ω^i), for all i

    let mut fracs: [F; FIELDS_PER_BLOB] = [BigNum::new(); FIELDS_PER_BLOB];
    for i in 0..FIELDS_PER_BLOB {
        let inv_denom = inv_denoms[i]; // 1 / (z - ω^i)

        fracs[i] = ys[i].__mul(inv_denom); // y_i / (z - ω^i)
    }
    fracs
}

unconstrained fn __field_to_bytes(x: Field) -> [u8; 32] {
    let x_bytes_slice = x.to_be_bytes(32);
    let mut x_bytes = [0; 32];
    for i in 0..32 {
        x_bytes[i] = x_bytes_slice[i];
    }
    x_bytes
}

unconstrained fn __field_to_bignum(x: Field) -> F {
    let x_bytes = __field_to_bytes(x);

    BigNum::from_byte_be(x_bytes)
    // TODO: use this once the name has been updated in the bignum lib:
    // BigNum::from_be_bytes(x_bytes)
}

unconstrained fn __field_to_bignum_limbs(x: Field) -> [Field; 3] {
    __field_to_bignum(x).limbs
}

// Only works for bignums with modulus larger than the BN Fr size (which is true
// for the bls12-381 Fr field).
fn field_to_bignum(x: Field) -> F {
    let __x_limbs = __field_to_bignum_limbs(x);

    let mut check = __x_limbs[3 - 1];
    for i in 1..3 {
        check *= LIMB_MAX;
        check += __x_limbs[3 - i - 1];
    }
    assert(check == x);

    BigNum { limbs: __x_limbs }
}

// DANGER: this assumes the input bignum is <= the Noir field size.
// Only use this if you _know_ the data being passed in is small enough.
//
// Or actually, maybe it's not unsafe, if Field catches overflows?
fn unsafe_bignum_to_field(x: F) -> Field {
    let mut result: Field = 0;
    result += x.limbs[3 - 1];
    for i in 1..3 {
        result *= LIMB_MAX;
        result += x.limbs[3 - i - 1];
    }
    result
}

fn bignum_to_bytes(x: F) -> [u8] {
    let limb_0_bytes: [u8] = x.limbs[0].to_be_bytes(15);
    let limb_1_bytes: [u8] = x.limbs[1].to_be_bytes(15);
    let limb_2_bytes: [u8] = x.limbs[2].to_be_bytes(2);
    let out = limb_2_bytes.append(limb_1_bytes).append(limb_0_bytes);
    std::static_assert(out.len() == 32, "bad byte decomposition of bignum");
    out
}

// fn kzg_commitment_to_bytes(c: [Field; 2]) -> [u8] {
//     let limb_0_bytes: [u8] = x.limbs[0].to_be_bytes(32);
//     let limb_1_bytes: [u8] = x.limbs[1].to_be_bytes(16);

//     let out = limb_2_bytes.append(limb_1_bytes).append(limb_0_bytes);
//     std::static_assert(out.len() == 32, "bad byte decomposition of bignum");
//     out
// }

// DANGER: this assumes the input bignum is <= the Noir field size.
// Only use this if you _know_ the data being passed in is small enough.
//
// This is inefficient, in the sense that we discard ~1 bit of blob space per
// 255-bit blob field, when converting it to a 245-bit noir field. Over the whole blob,
// we end up discarding 1 bit * 4096 fields_per_blob = 512 bytes = 16 words of data.
// BUT, it is much simpler to do this than to reconstitute 4096 disparate bits across
// the whole blob into 16 words. Perhaps the more complex approach should only be
// taken once aztec blobs are sufficiently full?
fn unsafe_blob_to_fields(blob: [F; FIELDS_PER_BLOB]) -> [Field; FIELDS_PER_BLOB] {
    let mut blob_as_fields: [Field; FIELDS_PER_BLOB] = [0; FIELDS_PER_BLOB];
    for i in 0..FIELDS_PER_BLOB {
        blob_as_fields[i] = unsafe_bignum_to_field(blob[i]);
    }
    blob_as_fields
}

unconstrained fn __unsafe_extract_top_bit(x: F) -> (Field, Field) {
    let top_limb: Field = x.limbs[2];
    // The top_limb is at most 2 bytes (16 bits).
    // 0x8000 = 2^15 = 32768
    let top_bit: Field = (top_limb as u16 / 0x8000) as Field;
    let top_limb_with_top_bit_removed = top_limb - top_bit * 0x8000;
    (top_bit, top_limb_with_top_bit_removed)
}

// DANGER: it's named as "unsafe" because the caller MUST already have checked that
// each blob Field is formatted as (u1, Field). I.e. the "rhs" 254-bits should already
// fit within a Field. If the "rhs" 254 bits is larger than the field modulus,
// there will be an uncaught overflow of the 254-bits in the Field, resulting in
// an unintended tiny value.
//
// For efficiency, the top_bit is kept as a Field throughout.
fn unsafe_extract_top_bit(x: F) -> (Field, F) {
    let (top_bit, top_limb_with_top_bit_removed) = __unsafe_extract_top_bit(x);
    assert_eq(top_bit * 0x8000 + top_limb_with_top_bit_removed, x.limbs[2]);

    (top_bit, BigNum { limbs: [x.limbs[0], x.limbs[1], top_limb_with_top_bit_removed] })
}

fn blob_to_fields__tightly_packed(blob: [F; FIELDS_PER_BLOB]) -> [Field; NOIR_FIELDS_PER_BLOB] {
    let mut blob_as_fields: [Field; NOIR_FIELDS_PER_BLOB] = [0; NOIR_FIELDS_PER_BLOB];
    let mut top_bits: [Field; FIELDS_CARRYING_AN_EXTRA_BIT_PER_BLOB] = [0; FIELDS_CARRYING_AN_EXTRA_BIT_PER_BLOB];

    // We start with [F; 4096].
    // The first 4064 of these bls-fields have a 255th bit (counting from 1) which can contribute towards
    // new 254-bit noir fields. That is, we extract 4064 top-bits from the first 4064 of the 4096 bls-fields,
    // and reconstitute them into 4064 / 254 = 16 extra noir fields.
    // So we end up with 4096 + 16 = 4112 noir fields.

    // Here we compute top_bits[0:4064] and blob_as_fields[0:4064].
    for i in 0..FIELDS_CARRYING_AN_EXTRA_BIT_PER_BLOB {
        let (top_bit, field_with_top_bit_removed): (Field, F) = unsafe_extract_top_bit(blob[i]);
        top_bits[i] = top_bit;
        blob_as_fields[i] = unsafe_bignum_to_field(field_with_top_bit_removed);
    }
    // Here we compute blob_as_fields[4064:4096].
    for i in FIELDS_CARRYING_AN_EXTRA_BIT_PER_BLOB..FIELDS_PER_BLOB {
        blob_as_fields[i] = unsafe_bignum_to_field(blob[i]);
    }
    // Here we compute blob_as_fields[4096:4112] from top_bits[0:4064]
    for i in FIELDS_PER_BLOB..NOIR_FIELDS_PER_BLOB {
        // the top_bits are assumed to be big-endian bit arrays:
        let mut reconstituted_field = top_bits[0];
        for j in 1..254 {
            let k = (i - FIELDS_PER_BLOB) * 254 + j;
            reconstituted_field *= 2;
            reconstituted_field += top_bits[k];
            // std::as_witness(reconstituted_field); // this was costing 4048 gates
        }
        blob_as_fields[i] = reconstituted_field;
    }
    blob_as_fields
}

// TODO: We'll want to hash this data
// in an arrangement which makes sense to the aztec protocol. THink about this more.
fn hash_blob(blob: [F; FIELDS_PER_BLOB]) -> Field {
    // let mut blob_as_fields = unsafe_blob_to_fields(blob);
    let mut blob_as_fields = blob_to_fields__tightly_packed(blob);
    let hash = poseidon2::Poseidon2::hash(blob_as_fields, blob_as_fields.len());
    hash
}

fn hash_kzg_commitment(kzg_commitment: [Field; 2]) -> Field {
    let hash = poseidon2::Poseidon2::hash(kzg_commitment, kzg_commitment.len());
    hash
}

fn compute_challenge(blob: [F; FIELDS_PER_BLOB], kzg_commitment: [Field; 2]) -> Field {
    let kzg_commitment_hash = hash_kzg_commitment(kzg_commitment);
    let blob_hash = hash_blob(blob);
    // let blob_hash = 1;
    let challenge = poseidon2::Poseidon2::hash([blob_hash, kzg_commitment_hash], 2);
    challenge
}

// ~500k constraints. 30 MINUTES TO COMPILE (due to all the brillig)!
//
// Note: the kzg_commitment is technically a BLS12-381 point in (Fq, Fq), but
// we haven't implemented Fq; only Fr, and we don't actually need to operate on it;
// we just need the bits of data. So we've simply encoded it as fitting inside a
// [Field; 2], since two 254-bit fields more-than covers 381+1=382 bits.
fn main(blob: [F; FIELDS_PER_BLOB], kzg_commitment: [Field; 2]) -> pub (Field, F, [Field; 2]) {
    let challenge_z: Field = compute_challenge(blob, kzg_commitment);
    let challenge_z_as_bignum: F = field_to_bignum(challenge_z);

    let y: F = barycentric_evaluate_blob_at_z(challenge_z_as_bignum, blob);
    // let y = challenge_z_as_bignum; // dummy constraint for when we want to skip the barycentric stuff in testing.

    // let challenge_z_as_bytes: [u8] = challenge_z.to_be_bytes(32);
    // let y_as_bytes: [u8] = bignum_to_bytes(y);
    // let kzg_commitment_as_bytes: [u8] = ()

    // TODO: this return data needs to be TIGHTLY PACKED into bytes.
    // TODO: then those bytes need to be sha256-hashed, to produce a single value that can be sent to ethereum for cheap snark verification. On ethereum, the bytes will be sent along with the sha256-hash of the bytes. The bytes will be used in the point evaluation precompile. The sha256-hash will form a part of the public inputs of the zk-snark proof.
    (challenge_z, y, kzg_commitment)
}

/**
 *                    ___d-1
 *         z^d - 1    \            ω^i
 * p(z) = --------- . /   y_i . ---------
 *            d      /____       z - ω^i
 *                    i=0
 *
 * p(z) = factor . sum( y_i . num / denom )
 *
 *
 * where d = 4096
 *
 * Precompute:
 * - The d roots of unity ω^i (plus maybe their negatives for z - ω^i computations).
 * - (1 / d)
 *
 * @param z
 * @param ys - the many y_i's of the blob.
 *
 * @return y = p(z)
 */
fn barycentric_evaluate_blob_at_z(z: F, ys: [F; FIELDS_PER_BLOB]) -> F {
    // z ^ D:

    let mut t1 = z.__mul(z);

    BigNum::evaluate_quadratic_expression([[z]], [[false]], [[z]], [[false]], [t1], [true]);

    let mut t2: F = BigNum::new();
    for _i in 0..LOG_FIELDS_PER_BLOB - 1 {
        t2 = t1.__mul(t1);

        // GRATUITOUS USAGE OF as_witness, LIKE THROWING DARTS AT A DARTBOARD AND HOPING THIS HELPS
        std::as_witness(t2.limbs[0]);
        std::as_witness(t2.limbs[1]);
        std::as_witness(t2.limbs[2]);

        BigNum::evaluate_quadratic_expression([[t1]], [[false]], [[t1]], [[false]], [t2], [true]);

        t1 = t2;
        std::as_witness(t1.limbs[0]);
        std::as_witness(t1.limbs[1]);
        std::as_witness(t1.limbs[2]);
    }

    let z_pow_d = t1;

    // factor:

    let one: F = BigNum::one();

    t1 = z_pow_d.__sub(one);
    std::as_witness(t1.limbs[0]);
    std::as_witness(t1.limbs[1]);
    std::as_witness(t1.limbs[2]);

    let factor = t1.__mul(D_INV);

    // (z_pow_d - one) * (D_INV) - factor = 0
    // z_pow_d * D_INV - D_INV - factor = 0
    BigNum::evaluate_quadratic_expression(
        [[z_pow_d]],
        [[false]],
        [[D_INV]],
        [[false]],
        [factor, D_INV],
        [true, true]
    );

    // This version doesn't work:
    // BigNum::evaluate_quadratic_expression(
    //     [[z_pow_d, one]],
    //     [[false, true]],
    //     [[D_INV]],
    //     [[false]],
    //     [factor],
    //     [true]
    // );

    // sum:

    let mut sum: F = BigNum::new();

    // Making a call to this function causes a "stack too deep" error, so I've put the body of that function here, instead:
    // let fracs = __compute_fracs(z, ys); // { y_i / (z - ω^i) }

    /**
      *
      * Note: it's more efficient (saving 30k constraints) to compute:
      *    ___d-1
      *    \     /    y_i    \
      *    /    |  ---------  | . ω^i
      *   /____  \  z - ω^i  /
      *    i=0
      *            ^^^^^^^^^
      *              frac
      *
      * ... than to compute:
      *
      *    ___d-1
      *    \          /    ω^i    \
      *    /   y_i . |  ---------  |
      *   /____       \  z - ω^i  /
      *    i=0
      *
      * perhaps because all the ω^i terms are constant witnesses?
      *
      */

    //*****************************************************************
    // This section is only needed because `__compute_fracs` isn't working (stack too deep error).

    let mut fracs: [F; FIELDS_PER_BLOB] = [BigNum::new(); FIELDS_PER_BLOB]; // y_i / (z - ω^i), for all i

    let mut denoms = [BigNum::new(); FIELDS_PER_BLOB];
    for i in 0..FIELDS_PER_BLOB {
        denoms[i] = z.__add(NEGATIVE_ROOTS[i]); // (z - ω^i)
    }

    // If you're seeing a `bug` warning for this line, I think it's fine.
    // Ideally, we'd be using `__compute_fracs`, anyway, but we're getting a "stack too deep" error.
    let inv_denoms = __batch_invert_impl(denoms); // 1 / (z - ω^i), for all i

    for i in 0..FIELDS_PER_BLOB {
        let num = ys[i];
        let inv_denom = inv_denoms[i]; // 1 / (z - ω^i)

        let frac = num.__mul(inv_denom); // y_i * (1 / (z - ω^i))

        fracs[i] = frac; // y_i / (z - ω^i)
        std::as_witness(fracs[i].limbs[0]);
        std::as_witness(fracs[i].limbs[1]);
        std::as_witness(fracs[i].limbs[2]);

        //End of section that is only needed because `__compute_fracs` isn't working
        //*****************************************************************

        // frac <-- ys[i] / (z + neg_roots[i])
        // frac * (z + neg_roots[i]) - ys[i] = 0
        BigNum::evaluate_quadratic_expression(
            [[fracs[i]]],
            [[false]],
            [[z, NEGATIVE_ROOTS[i]]],
            [[false, false]],
            [ys[i]],
            [true]
        );
    }

    // OK so...we can add multiple product terms into a sum...but I am not sure how many!
    // we are computing 254 * 254 bit products and we need to ensure each product limb doesn't overflow
    // each limb is 120 bits => 120 * 120 = 240 bits.
    // however when computing a mul we add up to 5 product terms into a single field element => 243 bits (ish)
    // when we do a modular reduction we validate that a field element >> 120 bits is less than 2^{126} which implies we have 246 bits to play with
    // which implies...we can accomodate up to EIGHT additions of product terms before we risk overflowing
    // (this is really messy! I never considered the case of giant linear sequences of products)
    let mut sum: F = BigNum::new();
    /**
     * Seeking:
     *                    ___d-1
     *                    \            ω^i
     *              sum = /   y_i . ---------
     *                   /____       z - ω^i
     *                    i=0
     */
    let NUM_PARTIAL_SUMS = FIELDS_PER_BLOB / 8;
    for i in 0..NUM_PARTIAL_SUMS {
        let mut partial_sum: F = BigNum::new();
        let mut lhs: [F; 8] = [BigNum::new(); 8];
        let mut rhs = lhs;

        /**
         * Seeking:
         *                    ___i*8 + 7
         *                    \            ω^k
         *      partial_sum = /   y_k . ---------
         *                   /____       z - ω^k
         *                    k=i*8 + 0
         */

        for j in 0..8 {
            let k = i * 8 + j;
            lhs[j] = ROOTS[k]; // ω^k
            rhs[j] = fracs[k]; // y_k / (z - ω^k)
            std::as_witness(lhs[j].limbs[0]);
            std::as_witness(lhs[j].limbs[1]);
            std::as_witness(lhs[j].limbs[2]);
            std::as_witness(rhs[j].limbs[0]);
            std::as_witness(rhs[j].limbs[1]);
            std::as_witness(rhs[j].limbs[2]);

            // y_k * ( ω^k / (z - ω^k) )
            let summand = ROOTS[k].__mul(fracs[k]);

            // partial_sum + ( y_k * ( ω^k / (z - ω^k) ) -> partial_sum
            partial_sum = partial_sum.__add(summand);
            std::as_witness(partial_sum.limbs[0]);
            std::as_witness(partial_sum.limbs[1]);
            std::as_witness(partial_sum.limbs[2]);
        }

        /**
         * Seeking:
         *                    ___i*8 - 1              ___i*8 + 7
         *                    \            ω^i        \            /    y_k    \
         *      sum_out   =   /   y_i . ---------  +  /     ω^k . |  ---------  |
         *                   /____       z - ω^i     /____         \  z - ω^k  /
         *                       0                    k = i*8
         *                   ^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
         *                          sum                         partial_sum
         *
         * ... that is:
         *
         *                    ___i*8 - 1              ___ 7
         *                    \            ω^i        \
         *      sum_out   =   /   y_i . ---------  +  /   lhs[j] . rhs[j]
         *                   /____       z - ω^i     /____
         *                       0                    j = 0
         *                   ^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^^
         *                          sum                   partial_sum
         */

        let mut sum_out = sum.__add(partial_sum);

        std::as_witness(sum_out.limbs[0]);
        std::as_witness(sum_out.limbs[1]);
        std::as_witness(sum_out.limbs[2]);

        //    sum_out <- sum + (lhs[0] * rhs[0] + ... + lhs[7] * rhs[7])
        // => (lhs[0] * rhs[0] + ... + lhs[7] * rhs[7]) + sum - sum_out == 0
        BigNum::evaluate_quadratic_expression(
            [[lhs[0]], [lhs[1]], [lhs[2]], [lhs[3]], [lhs[4]], [lhs[5]], [lhs[6]], [lhs[7]]],
            [[false], [false], [false], [false], [false], [false], [false], [false]],
            [[rhs[0]], [rhs[1]], [rhs[2]], [rhs[3]], [rhs[4]], [rhs[5]], [rhs[6]], [rhs[7]]],
            [[false], [false], [false], [false], [false], [false], [false], [false]],
            [sum, sum_out],
            [false, true]
        );

        sum = sum_out;
        std::as_witness(sum.limbs[0]);
        std::as_witness(sum.limbs[1]);
        std::as_witness(sum.limbs[2]);
    }

    // y:

    let y = factor.__mul(sum);

    //    y <- factor * sum
    // => factor * sum - y == 0
    BigNum::evaluate_quadratic_expression([[factor]], [[false]], [[sum]], [[false]], [y], [true]);

    println(y);

    y
}

// Need to wait for https://github.com/noir-lang/noir-bignum/pull/9 for this to work.
// nargo test --show-output test_main
// #[test]
// fn test_main() {
//     let mut blob: [F; FIELDS_PER_BLOB] = [BigNum::new(); FIELDS_PER_BLOB];

//     blob[0] = BigNum { limbs: [0x1234, 0, 0] };
//     blob[1] = BigNum { limbs: [0xabcd, 0, 0] };
//     blob[2] = BigNum { limbs: [0x69, 0, 0] };

//     let kzg_commitment_in = [1, 2]; // this is made-up nonsense.

//     let (challenge_z, y, kzg_commitment) = main(blob, kzg_commitment_in);

//     println(challenge_z);
//     println(y);
//     println(kzg_commitment);
// }

#[test]
fn test_print() {
    let x = 4;
    println(f"x: {x}");
}

// Need to wait for https://github.com/noir-lang/noir-bignum/pull/9 for this to work.
// #[test]
// fn test_bignum_conversions() {
//     let x = 1000;
//     let x_bignum = __field_to_bignum(x);
//     println(f"x_bignum: {x_bignum}");
// }

// nargo test --show-output test_barycentric
#[test]
fn test_barycentric() {
    let z: F = BigNum { limbs: [2, 0, 0] };

    // many y's form a blob:
    let mut ys: [F; FIELDS_PER_BLOB] = [BigNum::new(); FIELDS_PER_BLOB];

    ys[0] = BigNum { limbs: [0x1234, 0, 0] };
    ys[1] = BigNum { limbs: [0xabcd, 0, 0] };
    ys[2] = BigNum { limbs: [0x69, 0, 0] };

    // evaluate the blob at z = 2 to yield y:
    let y = barycentric_evaluate_blob_at_z(z, ys);

    let mut expected_y: [Field; 3] = [0; 3];
    if (FIELDS_PER_BLOB == 4096) {
        // Computed with the eth consensus specs py lib
        expected_y = [
            0x0c62e352a428e8e9842eadc1c106bd,
            0x902c5b4968d755b6f49c0231e15af8,
            0x00049a
        ];
        // Also computed with cKzg, in the typescript tests:
        // 0x049a902c5b4968d755b6f49c0231e15af80c62e352a428e8e9842eadc1c106bd
    }
    if (FIELDS_PER_BLOB == 8) {
        // Computed with the eth consensus specs py lib (after hacking it to cope with blobs of size 8 instead of 4096):
        expected_y = [
            0xb04cdea4304000053abffffffb203a,
            0x0000000002e30785c8afa4496f8e38,
            0x000000
        ];
    }
    assert(y.limbs == expected_y);
}

// Helper function used to populate the hard-coded double_modulus value in the bls12381Fr.nr file in the bignum library.
unconstrained fn compute_double_modulus() -> [Field; 3] {
    let two_p = [0x7b4805fffcb7fdfffffffe00000002, 0x4ea6533afa906673b0101343b00aa7, 0x00e7db];
    let NUM_LIMBS = 3; // must be >= 3
    let two_pow_120 = 2.pow_32(120);
    let mut double_modulus: [Field; 3] = [0; 3];

    double_modulus[0] = two_p[0] + two_pow_120;
    for i in 1..NUM_LIMBS - 1 {
        double_modulus[i] = two_p[i] + two_pow_120 - 1;
    }
    double_modulus[NUM_LIMBS - 1] = two_p[NUM_LIMBS - 1] - 1;
    double_modulus
}

// nargo test --show-output test_compute_double_modulus
#[test]
fn test_compute_double_modulus() {
    println(compute_double_modulus());
}

// nargo test --show-output test_compute_d_inv
#[test]
fn test_compute_d_inv() {
    let D_INV = D.__invmod();
    println(D_INV);
}

