use crate::blob::{
    blob_public_inputs::{BlobCommitment, BlobPublicInputs, BlockBlobPublicInputs},
    config::{D_INV, LOG_FIELDS_PER_BLOB, ROOTS},
};

use bigint::{BigNum, BLS12_381_Fr as F};
use std::ops::{Mul, Neg};
use crate::types::{
    abis::sponge_blob::SpongeBlob,
    constants::{BLOBS_PER_BLOCK, FIELDS_PER_BLOB},
    hash::poseidon2_hash_subarray,
    traits::Empty,
    utils::arrays::array_splice,
};

// TODO(MW): remove pub(crate)when fully moved to batching
pub(crate) fn convert_blob_fields(
    blob_as_fields: [Field; FIELDS_PER_BLOB],
) -> [F; FIELDS_PER_BLOB] {
    let mut blob: [F; FIELDS_PER_BLOB] = [F::zero(); FIELDS_PER_BLOB];
    for i in 0..FIELDS_PER_BLOB {
        blob[i] = F::from(blob_as_fields[i]);
    }
    blob
}

pub(crate)fn check_block_blob_sponge(
    blobs_as_fields: [Field; FIELDS_PER_BLOB * BLOBS_PER_BLOCK],
    mut sponge_blob: SpongeBlob,
) -> Field {
    // Check that we haven't overfilled the blobs
    assert(
        sponge_blob.expected_fields <= FIELDS_PER_BLOB * BLOBS_PER_BLOCK,
        "Attempted to overfill blobs",
    );
    // Check that the blob is full
    assert(
        sponge_blob.expected_fields == sponge_blob.fields,
        "Incorrect number of tx effects added to blob",
    );
    let sponge_hash = sponge_blob.squeeze();
    let hash = poseidon2_hash_subarray(blobs_as_fields, sponge_blob.fields);
    assert(hash == sponge_hash, "Mismatched hashed tx effects");

    sponge_hash
}

// TODO(MW): remove pub(crate)when fully moved to batching
pub(crate) fn compute_challenge(
    hashed_blobs_fields: Field,
    kzg_commitment: BlobCommitment,
) -> Field {
    let preimage = [hashed_blobs_fields, kzg_commitment.inner[0], kzg_commitment.inner[1]];
    let challenge = poseidon::poseidon2::Poseidon2::hash(preimage, 3);
    challenge
}

// Note: the kzg_commitment is technically a BLS12-381 point in (Fq, Fq), but
// we don't actually need to operate on it so we've simply encoded it as fitting inside a
// [Field; 2], since two 254-bit fields more-than covers 381+1=382 bits.
// See yarn-project/foundation/src/blob/index.ts -> commitmentToFields() for encoding
// TODO(MW): remove pub(crate)when fully moved to batching
pub(crate) fn evaluate_blob(
    blob_as_fields: [Field; FIELDS_PER_BLOB],
    kzg_commitment: BlobCommitment,
    hashed_blobs_fields: Field,
) -> BlobPublicInputs {
    let challenge_z: Field = compute_challenge(hashed_blobs_fields, kzg_commitment);
    let challenge_z_as_bignum = F::from(challenge_z);
    let blob = convert_blob_fields(blob_as_fields);

    let y: F = barycentric_evaluate_blob_at_z(challenge_z_as_bignum, blob);

    BlobPublicInputs { z: challenge_z, y, kzg_commitment }
}

// Evaluates each blob required for a block
pub(crate)fn evaluate_blobs(
    blobs_as_fields: [Field; FIELDS_PER_BLOB * BLOBS_PER_BLOCK],
    kzg_commitments: [BlobCommitment; BLOBS_PER_BLOCK],
    mut sponge_blob: SpongeBlob,
) -> BlockBlobPublicInputs {
    // Note that with multiple blobs per block, each blob uses the same hashed_blobs_fields in:
    // challenge_z = H(hashed_blobs_fields, kzg_commitment[0], kzg_commitment[1])
    // This is ok, because each commitment is unique to the blob, and we need hashed_blobs_fields to encompass
    // all fields in the blob, which it does.
    let hashed_blobs_fields = check_block_blob_sponge(blobs_as_fields, sponge_blob);
    let mut result = BlockBlobPublicInputs::empty();
    for i in 0..BLOBS_PER_BLOCK {
        let single_blob_fields = array_splice(blobs_as_fields, i * FIELDS_PER_BLOB);
        result.inner[i] =
            evaluate_blob(single_blob_fields, kzg_commitments[i], hashed_blobs_fields);
        if (result.inner[i].is_zero()) & (single_blob_fields[0] == 0) {
            // We use empty PIs for empty blobs, to make it simpler to verify on L1.
            // Since our fields come from the base rollup, we know they are tightly packed
            // and should contain no 0 values among valid values => single_blob_fields[0] == 0.
            result.inner[i] = BlobPublicInputs::empty();
        }
    }
    result
}

/**
 *                    ___d-1
 *         z^d - 1    \            omega^i
 * p(z) = --------- . /   y_i . ---------
 *            d      /____       z - omega^i
 *                    i=0
 *
 * p(z) = factor . sum( y_i . num / denom )
 *
 *
 * where d = 4096
 *
 * Precompute:
 * - The d roots of unity omega^i (plus maybe their negatives for z - omega^i computations).
 * - (1 / d)
 *
 * @param z
 * @param ys - the many y_i's of the blob.
 *
 * @return y = p(z)
 * TODO(MW): remove pub(crate)when fully moved to batching
 */
pub(crate) fn barycentric_evaluate_blob_at_z(z: F, ys: [F; FIELDS_PER_BLOB]) -> F {
    // Note: it's more efficient (saving 30k constraints) to compute:
    //    ___d-1
    //    \     /    y_i    \
    //    /    |  ---------  | . omega^i
    //   /____  \  z - omega^i  /
    //    i=0
    //            ^^^^^^^^^
    //              frac
    //
    // ... than to compute:
    //
    //    ___d-1
    //    \          /    omega^i    \
    //    /   y_i . |  ---------  |
    //   /____       \  z - omega^i  /
    //    i=0
    //
    // perhaps because all the omega^i terms are constant witnesses?
    let fracs = compute_fracs(z, ys);

    let sum = if !std::runtime::is_unconstrained() {
        // OK so...we can add multiple product terms into a sum...but I am not sure how many!
        // we are computing 254 * 254 bit products and we need to ensure each product limb doesn't overflow
        // each limb is 120 bits => 120 * 120 = 240 bits.
        // however when computing a mul we add up to 5 product terms into a single field element => 243 bits (ish)
        // when we do a modular reduction we validate that a field element >> 120 bits is less than 2^{126} which implies we have 246 bits to play with
        // which implies...we can accommodate up to EIGHT additions of product terms before we risk overflowing
        // (this is really messy! I never considered the case of giant linear sequences of products)

        // Seeking:
        //                    ___d-1
        //                    \            omega^i
        //              sum = /   y_i . ---------
        //                   /____       z - omega^i
        //                    i=0

        let NUM_PARTIAL_SUMS = FIELDS_PER_BLOB / 8;
        // Safety: This sum is checked by the following `F::evaluate_quadratic_expression` calls.
        let partial_sums: [F; FIELDS_PER_BLOB / 8] = unsafe { __compute_partial_sums(fracs) };

        // We split off the first term to check the initial sum

        //    partial_sums[0] <- (lhs[0] * rhs[0] + ... + lhs[7] * rhs[7])
        // => (lhs[0] * rhs[0] + ... + lhs[7] * rhs[7]) - partial_sums[0] == 0
        let lhs = [
            [ROOTS[0]], [ROOTS[1]], [ROOTS[2]], [ROOTS[3]], [ROOTS[4]], [ROOTS[5]], [ROOTS[6]],
            [ROOTS[7]],
        ];
        let rhs = [
            [fracs[0]], [fracs[1]], [fracs[2]], [fracs[3]], [fracs[4]], [fracs[5]], [fracs[6]],
            [fracs[7]],
        ];
        bigint::bignum::evaluate_quadratic_expression(
            lhs,
            [[false], [false], [false], [false], [false], [false], [false], [false]],
            rhs,
            [[false], [false], [false], [false], [false], [false], [false], [false]],
            [partial_sums[0]],
            [true],
        );
        for i in 1..NUM_PARTIAL_SUMS {
            // Seeking:
            //                    ___i*8 - 1              ___i*8 + 7
            //                    \            omega^i        \            /    y_k    \
            //      sum_out   =   /   y_i . ---------  +  /     omega^k . |  ---------  |
            //                   /____       z - omega^i     /____         \  z - omega^k  /
            //                       0                    k = i*8
            //                   ^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
            //                          sum                         partial_sum
            //
            // ... that is:
            //
            //                    ___i*8 - 1              ___ 7
            //                    \            omega^i        \
            //      sum_out   =   /   y_i . ---------  +  /   lhs[j] . rhs[j]
            //                   /____       z - omega^i     /____
            //                       0                    j = 0
            //                   ^^^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^^
            //                          sum                   partial_sum
            //

            //    partial_sums[i] <- partial_sums[i-1] + (lhs[8*i] * rhs[8*i] + ... + lhs[8*i + 7] * rhs[8*i + 7])
            // => (lhs[8*i] * rhs[8*i] + ... + lhs[8*i + 7] * rhs[8*i + 7]) + partial_sums[i-1] - partial_sums[i] == 0
            let linear_terms = [partial_sums[i - 1], partial_sums[i]];

            bigint::bignum::evaluate_quadratic_expression(
                /* lhs */ [
                    [ROOTS[i * 8 + 0]],
                    [ROOTS[i * 8 + 1]],
                    [ROOTS[i * 8 + 2]],
                    [ROOTS[i * 8 + 3]],
                    [ROOTS[i * 8 + 4]],
                    [ROOTS[i * 8 + 5]],
                    [ROOTS[i * 8 + 6]],
                    [ROOTS[i * 8 + 7]],
                ],
                [[false], [false], [false], [false], [false], [false], [false], [false]],
                /* rhs */
                [
                    [fracs[i * 8 + 0]],
                    [fracs[i * 8 + 1]],
                    [fracs[i * 8 + 2]],
                    [fracs[i * 8 + 3]],
                    [fracs[i * 8 + 4]],
                    [fracs[i * 8 + 5]],
                    [fracs[i * 8 + 6]],
                    [fracs[i * 8 + 7]],
                ],
                [[false], [false], [false], [false], [false], [false], [false], [false]],
                linear_terms,
                [false, true],
            );
        }

        // The final term in `partial_sums` is then the full sum.
        partial_sums[FIELDS_PER_BLOB / 8 - 1]
    } else {
        // There's no need to jump through hoops to check partial sums if we don't need to constrain the result.
        // We can just calculate the final sum.

        // Safety: We're running under the condition that `std::runtime::is_unconstrained()` is true.
        unsafe {
            __compute_sum(fracs)
        }
    };

    let factor = compute_factor(z);
    factor.mul(sum)
}

unconstrained fn __compute_factor_helper(z_pow_d: F) -> F {
    let one: F = F::one();
    z_pow_d.__sub(one).__mul(D_INV)
}

fn compute_factor(z: F) -> F {
    // z ^ D:
    let mut t = z.mul(z);

    for _ in 0..LOG_FIELDS_PER_BLOB - 1 {
        t = t.mul(t);
    }

    let z_pow_d = t;

    // Safety: We immediately check that this result is correct in the following
    // `F::evaluate_quadratic_expression` call.
    let factor = unsafe { __compute_factor_helper(z_pow_d) };

    // (z_pow_d - one) * (D_INV) - factor = 0
    // z_pow_d * D_INV - D_INV - factor = 0
    if !std::runtime::is_unconstrained() {
        bigint::bignum::evaluate_quadratic_expression(
            [[z_pow_d]],
            [[false]],
            [[D_INV]],
            [[false]],
            [factor, D_INV],
            [true, true],
        );
    }

    // This version doesn't work:
    // F::evaluate_quadratic_expression(
    //     [[z_pow_d, one]],
    //     [[false, true]],
    //     [[D_INV]],
    //     [[false]],
    //     [factor],
    //     [true]
    // );

    factor
}

unconstrained fn __compute_fracs(z: F, ys: [F; FIELDS_PER_BLOB]) -> [F; FIELDS_PER_BLOB] {
    let mut denoms = [F::zero(); FIELDS_PER_BLOB];
    for i in 0..FIELDS_PER_BLOB {
        denoms[i] = z.__sub(ROOTS[i]); // (z - omega^i)
    }
    let inv_denoms: [F; FIELDS_PER_BLOB] = bigint::bignum::batch_invert(denoms); // 1 / (z - omega^i), for all i
    // We're now done with `denoms` so we can reuse the allocated array to build `fracs`.
    let mut fracs: [F; FIELDS_PER_BLOB] = denoms; // y_i / (z - omega^i), for all i
    for i in 0..FIELDS_PER_BLOB {
        let num = ys[i];
        let inv_denom = inv_denoms[i]; // 1 / (z - omega^i)
        fracs[i] = num.__mul(inv_denom); // y_i * (1 / (z - omega^i))
    }

    fracs
}

fn compute_fracs(z: F, ys: [F; FIELDS_PER_BLOB]) -> [F; FIELDS_PER_BLOB] {
    // Safety: We immediately constrain these `fracs` to be correct in the following call
    // to `F::evaluate_quadratic_expression`.
    let mut fracs: [F; FIELDS_PER_BLOB] = unsafe { __compute_fracs(z, ys) };

    if !std::runtime::is_unconstrained() {
        for i in 0..FIELDS_PER_BLOB {
            // frac <-- ys[i] / (z + neg_roots[i])
            // frac * (z + neg_roots[i]) - ys[i] = 0
            bigint::bignum::evaluate_quadratic_expression(
                [[fracs[i]]],
                [[false]],
                [[z, ROOTS[i].neg()]],
                [[false, false]],
                [ys[i]],
                [true],
            );
        }
    }

    fracs
}

// TODO: Clean me
unconstrained fn __compute_partial_sums(fracs: [F; FIELDS_PER_BLOB]) -> [F; FIELDS_PER_BLOB / 8] {
    let mut partial_sums: [F; FIELDS_PER_BLOB / 8] = std::mem::zeroed();

    // Seeking:
    //                    ___i*8 + 7
    //                    \            omega^k
    //      partial_sum = /   y_k . ---------
    //                   /____       z - omega^k
    //                    k=i*8 + 0

    // Need to split off the first iteration.
    let mut partial_sum: F = F::zero();
    for i in 0..8 {
        // y_k * ( omega^k / (z - omega^k) )
        let summand = ROOTS[i].__mul(fracs[i]);

        // partial_sum + ( y_k * ( omega^k / (z - omega^k) ) -> partial_sum
        partial_sum = partial_sum.__add(summand);
    }
    partial_sums[0] = partial_sum;

    let NUM_PARTIAL_SUMS = FIELDS_PER_BLOB / 8;
    for i in 1..NUM_PARTIAL_SUMS {
        let mut partial_sum: F = partial_sums[i - 1];
        // Seeking:
        //                    ___i*8 + 7
        //                    \            omega^k
        //      partial_sum = /   y_k . ---------
        //                   /____       z - omega^k
        //                    k=i*8 + 0
        for j in 0..8 {
            let k = i * 8 + j;
            // y_k * ( omega^k / (z - omega^k) )
            let summand = ROOTS[k].__mul(fracs[k]);
            // partial_sum + ( y_k * ( omega^k / (z - omega^k) ) -> partial_sum
            partial_sum = partial_sum.__add(summand);
        }
        partial_sums[i] = partial_sum;
    }

    partial_sums
}

unconstrained fn __compute_sum(fracs: [F; FIELDS_PER_BLOB]) -> F {
    // Seeking:
    //                    ___d-1
    //                    \            omega^i
    //              sum = /   y_i . ---------
    //                   /____       z - omega^i
    //                    i=0

    let mut sum: F = F::zero();
    for i in 0..FIELDS_PER_BLOB {
        // y_k * ( omega^k / (z - omega^k) )
        let summand = ROOTS[i].__mul(fracs[i]);

        // partial_sum + ( y_k * ( omega^k / (z - omega^k) ) -> partial_sum
        sum = sum.__add(summand);
    }
    sum
}
