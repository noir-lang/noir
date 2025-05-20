use crate::{
    blob::{
        barycentric_evaluate_blob_at_z, check_block_blob_sponge, compute_challenge,
        convert_blob_fields,
    },
    blob_batching_public_inputs::{
        BatchingBlobCommitment, BlobAccumulationInputs, BlobAccumulatorPublicInputs, BLSPoint,
        compress_to_blob_commitment, FinalBlobBatchingChallenges,
    },
    blob_public_inputs::BlobCommitment,
};
use bigint::{BigNum, BLS12_381_Fr as F};
use types::{
    abis::sponge_blob::SpongeBlob,
    constants::{BLOBS_PER_BLOCK, FIELDS_PER_BLOB},
    traits::is_empty,
    utils::arrays::array_splice,
};

// Evaluates a single blob:
// - Evaluates the blob at shared challenge z and returns result y_i
// - Calculates this blob's challenge z_i (= H(H(blob_i), C_i))
fn evaluate_blob_for_batching(
    blob_as_fields: [Field; FIELDS_PER_BLOB],
    kzg_commitment: BatchingBlobCommitment,
    hashed_blobs_fields: Field,
    challenge_z: Field,
) -> (Field, F) {
    let challenge_z_as_bignum = F::from(challenge_z);
    let blob = convert_blob_fields(blob_as_fields);

    let y_i: F = barycentric_evaluate_blob_at_z(challenge_z_as_bignum, blob);
    let z_i: Field = compute_challenge(
        hashed_blobs_fields,
        // TODO(MW): At some point BatchingBlobCommitment will replace BlobCommitment and we won't need this silly conversion
        BlobCommitment { inner: kzg_commitment.to_compressed_fields() },
    );

    (z_i, y_i)
}

// Evaluates each blob required for a block:
// - Hashes all fields in the block's blobs (to use for the challenges z_i)
// - Compresses each of the blob's injected commitments (")
// - Evaluates each blob individually to find its challenge z_i & evaluation y_i
// - Updates the batched blob accumulator
pub fn evaluate_blobs_and_batch(
    blobs_as_fields: [Field; FIELDS_PER_BLOB * BLOBS_PER_BLOCK],
    kzg_commitments_points: [BLSPoint; BLOBS_PER_BLOCK],
    mut sponge_blob: SpongeBlob,
    final_blob_challenges: FinalBlobBatchingChallenges,
    start_accumulator: BlobAccumulatorPublicInputs,
) -> BlobAccumulatorPublicInputs {
    // See components.nr out_sponge definition as to why we copy here:
    let mut end_accumulator = start_accumulator;
    // Note that with multiple blobs per block, each blob uses the same hashed_blobs_fields in:
    // z_i = H(hashed_blobs_fields, kzg_commitment[0], kzg_commitment[1])
    // This is ok, because each commitment is unique to the blob, and we need hashed_blobs_fields to encompass
    // all fields in the blob, which it does.
    let hashed_blobs_fields = check_block_blob_sponge(blobs_as_fields, sponge_blob);
    for i in 0..BLOBS_PER_BLOCK {
        let single_blob_fields = array_splice(blobs_as_fields, i * FIELDS_PER_BLOB);
        let c_i = compress_to_blob_commitment(kzg_commitments_points[i]);
        let (z_i, y_i) = evaluate_blob_for_batching(
            single_blob_fields,
            c_i,
            hashed_blobs_fields,
            final_blob_challenges.z,
        );
        if !(y_i.is_zero()) & !(single_blob_fields[0] == 0) {
            // Only accumulate if the blob is non empty
            if is_empty(end_accumulator) {
                // TODO(MW): move this if is_empty() statement to .accumulate()?
                // Init only if accumulator is empty:
                //  - This will be checked in root, where the left input's start acc will be constrained to be zero
                //  - No other accs can be zero since each block_merge checks left's end acc == right's start acc
                end_accumulator = BlobAccumulatorPublicInputs::init(
                    BlobAccumulationInputs { z_i, y_i, c_i },
                    final_blob_challenges.gamma,
                );
            } else {
                end_accumulator = end_accumulator.accumulate(
                    BlobAccumulationInputs { z_i, y_i, c_i },
                    final_blob_challenges.gamma,
                );
            }
        }
    }
    end_accumulator
}

mod tests {
    use crate::{
        blob_batching::evaluate_blobs_and_batch,
        blob_batching_public_inputs::{
            BatchingBlobCommitment, BlobAccumulatorPublicInputs, compress_to_blob_commitment,
        },
    };
    use crate::blob_batching_public_inputs::FinalBlobBatchingChallenges;
    use bigcurve::{BigCurveTrait, curves::bls12_381::BLS12_381 as BLSPoint};
    use bigint::{BigNum, BLS12_381_Fr as F};
    use types::{
        abis::sponge_blob::SpongeBlob,
        constants::{BLOBS_PER_BLOCK, FIELDS_PER_BLOB},
        hash::sha256_to_field,
        tests::utils::pad_end,
        traits::Empty,
    };

    // All hardcoded values in this test are taken from yarn-project/foundation/src/blob/blob_batching.test.ts -> 'should construct and verify a batched blob of 400 items'
    #[test]
    unconstrained fn test_400_batched() {
        // We evaluate 1 blob of 400 items using the batch methods.
        // This ensures a block with a single blob will work:
        let mut blob: [Field; FIELDS_PER_BLOB] = [0; FIELDS_PER_BLOB];
        for i in 0..400 {
            blob[i] = 3;
        }
        let mut sponge_blob = SpongeBlob::new(400);
        sponge_blob.absorb(blob, 400);
        let kzg_commitment_in = BatchingBlobCommitment::from_limbs(
            [
                0xa971c7e8d8292be943d05bccebcfea,
                0xcddefc3721a54895a7a45e77504dd1,
                0x5fe972914ba3616033e2748bbaa6db,
                0x12803d,
            ],
            [
                0x71bde5210b6cae1530202c8a928127,
                0x5e7d987fb4afc5bcee960c6fc0628c,
                0x64801e9aff2901eb6916e65c51f280,
                0x1996ca,
            ],
        )
            .point;
        let kzg_commitments_in =
            [kzg_commitment_in, BLSPoint::point_at_infinity(), BLSPoint::point_at_infinity()];

        let final_challenges = FinalBlobBatchingChallenges {
            // = z_0
            z: 0x135d767e8b86b949d264be7a6b71d257c538893f3cef60c95d76ba420df18c3c,
            // = H(y_0, z_0)
            gamma: F::from_limbs([
                0xda0ebb0c577c62d5954852cf7a8863,
                0xaac05db8dabf148f011d29f2d308e4,
                0x0b28,
            ]),
        };
        // Evaluation
        let res = evaluate_blobs_and_batch(
            pad_end(blob, 0),
            kzg_commitments_in,
            sponge_blob,
            final_challenges,
            BlobAccumulatorPublicInputs::empty(),
        );
        let final_acc = res.finalize_and_validate(final_challenges);

        assert_eq(final_acc.z, final_challenges.z);
        // Since i = 1, gamma_pow = gamma^1 = gamma:
        assert_eq(res.gamma_pow, final_challenges.gamma);

        // y is a BLS field with value 0x212c4f0c0ee5e7dd037110686a4639d191dde7b57ab99b51e4b06e7d827b6c4c
        let expected_y: F = F::from_limbs([
            0xdde7b57ab99b51e4b06e7d827b6c4c,
            0x4f0c0ee5e7dd037110686a4639d191,
            0x212c,
        ]);
        assert_eq(final_acc.y, expected_y);

        let blob_commitment = compress_to_blob_commitment(kzg_commitments_in[0]);

        // Since i = 1, blob_commitments_hash is just the sha256 hash of the single (compressed) commitment
        let expected_blob_commitments_hash = sha256_to_field(blob_commitment.compressed);
        assert_eq(final_acc.blob_commitments_hash, expected_blob_commitments_hash);

        // Since i = 1, C = gamma^0 * C_0 = C_0
        assert_eq(final_acc.c, blob_commitment.to_compressed_fields());
    }

    // All hardcoded values in this test are taken from yarn-project/foundation/src/blob/blob_batching.test.ts -> 'should construct and verify a batch of 3 full blobs'
    #[test]
    unconstrained fn test_full_blobs_batched() {
        // Fill three blobs completely with different values (to avoid a constant polynomial)
        let mut blob: [Field; FIELDS_PER_BLOB * BLOBS_PER_BLOCK] =
            [0; FIELDS_PER_BLOB * BLOBS_PER_BLOCK];
        for j in 0..BLOBS_PER_BLOCK {
            for i in 0..FIELDS_PER_BLOB {
                blob[j * FIELDS_PER_BLOB + i] = ((j + 3) * (i + 1)) as Field;
            }
        }
        // Absorb the values into a sponge
        let mut sponge_blob = SpongeBlob::new(FIELDS_PER_BLOB * BLOBS_PER_BLOCK);
        sponge_blob.absorb(blob, FIELDS_PER_BLOB * BLOBS_PER_BLOCK);
        // Init. injected values:
        // - Commitments are injected and checked for correctness on L1 via acc.v
        let kzg_commitments_in = [
            BatchingBlobCommitment::from_limbs(
                [
                    0x2627fc88755984d7f002e5ef0e6b3e,
                    0x0ea98f6a26672e17f919eb020b00ee,
                    0xea6e5173f2ef1bedbb07bfa9ac6ed8,
                    0x01c6e6,
                ],
                [
                    0xaa96f04ba1d419683f218cc4f15a3f,
                    0x8887c5e719583b765309e4b3d18752,
                    0x9ff512de37b8582f7167fdfbb29539,
                    0x18f531,
                ],
            )
                .point,
            BatchingBlobCommitment::from_limbs(
                [
                    0x71556bb7217816fbb3f822fc873740,
                    0x9c57d93d7fd33a388e13e95cfdba95,
                    0x4f0ddbdc9d6a3653cd825ebd9f5730,
                    0x12324e,
                ],
                [
                    0xabf9f60fc773ef1802a706d6b170a4,
                    0x788f9000166d54151ac05df44e63be,
                    0x0a8b45ead129885bb12837fb59033b,
                    0x12aadd,
                ],
            )
                .point,
            BatchingBlobCommitment::from_limbs(
                [
                    0x69afb94a09e713e7fb94e26f33c3ed,
                    0x8161293f65480c3b7bad57aaef1984,
                    0xc34d68dc32d1ecd46f46ec4c969bb1,
                    0x0d97ef,
                ],
                [
                    0x9837a79d9fa4d0370198419b273360,
                    0x9e7340f07732e2cb3d51db22b1dcb3,
                    0x8285e8cad42f634bb51ad7d2c68a12,
                    0x07db3c,
                ],
            )
                .point,
        ];

        let final_challenges = FinalBlobBatchingChallenges {
            // - The final z value is injected and checked for correctness in root (see below final_acc)
            z: 0x02d6a54e591ada73e5eea35188a02ac87779f4293ea3e7d675fa50ae7ff332ce,
            // - The final gamma value is injected and checked for correctness in root (see below final_acc)
            gamma: F::from_limbs([
                0x281287a8d44071d216177e06a02327,
                0x16571aa3dcfef75c2447c705c6c68a,
                0x16f2,
            ]),
        };
        // Init. the accumulator
        let start_acc = BlobAccumulatorPublicInputs::empty();
        // Evaluate all three blobs and iteratively accumulate the results
        let output = evaluate_blobs_and_batch(
            blob,
            kzg_commitments_in,
            sponge_blob,
            final_challenges,
            start_acc,
        );
        // Finalize the output (actually done in the root circuit)
        let final_acc = output.finalize_and_validate(final_challenges);

        assert_eq(final_acc.z, final_challenges.z);
        assert_eq(
            output.gamma_pow,
            final_challenges.gamma.__pow(F::from(BLOBS_PER_BLOCK as Field)),
        );

        // y is a BLS Fr field with value 0x0cd2fd9a46ba70fd7f212d08ec7283024b0b1ff9446b1f78a482fb7443e49b57
        let expected_y = F::from_limbs([
            0x0b1ff9446b1f78a482fb7443e49b57,
            0xfd9a46ba70fd7f212d08ec7283024b,
            0x0cd2,
        ]);

        // C is a BLS point with value:
        // x: 0x0f2f5f62cc6c3ab4c1ac1abcb9da9677e12796a76064f68c0d4f659f25a046a6d42616100269935afcb1b98c85d5e93e,
        // y: 0x0af1e4abfa449daf65201c2b24507b1058d8ea9bf82ff948a1d01912615c4a8e507160da282e6c41bab917c868923254,
        let expected_c = BatchingBlobCommitment::from_limbs(
            [
                0x2616100269935afcb1b98c85d5e93e,
                0x96a76064f68c0d4f659f25a046a6d4,
                0x62cc6c3ab4c1ac1abcb9da9677e127,
                0x0f2f5f,
            ],
            [
                0x7160da282e6c41bab917c868923254,
                0xea9bf82ff948a1d01912615c4a8e50,
                0xabfa449daf65201c2b24507b1058d8,
                0x0af1e4,
            ],
        );

        // blob_commitments_hash is a BN Fr field with value 0x00d2f7bffbc5a9008207a188e348e753087f54557a686efd7f74c90cac52a9a1
        let expected_blob_commitments_hash =
            0xd2f7bffbc5a9008207a188e348e753087f54557a686efd7f74c90cac52a9a1;

        assert_eq(final_acc.y, expected_y);
        assert_eq(final_acc.c, expected_c.to_compressed_fields());
        assert_eq(final_acc.blob_commitments_hash, expected_blob_commitments_hash);
    }
}
