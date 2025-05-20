use crate::blob::{
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
use crate::types::{
    abis::sponge_blob::SpongeBlob,
    constants::{BLOBS_PER_BLOCK, FIELDS_PER_BLOB},
    traits::is_empty,
    utils::arrays::array_splice,
};
use bigint::{BigNum, BLS12_381_Fr as F};

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
pub(crate) fn evaluate_blobs_and_batch(
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
