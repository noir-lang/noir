use dep::types::{
    hash::{compute_tx_logs_hash, compute_tx_note_logs_hash},
    abis::{
    accumulated_data::{CombinedAccumulatedData, PublicAccumulatedData},
    log_hash::{LogHash, ScopedLogHash}, note_hash::ScopedNoteHash, nullifier::Nullifier,
    public_data_update_request::PublicDataUpdateRequest, side_effect::Ordered
},
    constants::{
    MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, MAX_UNENCRYPTED_LOGS_PER_TX, MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
    MAX_ENCRYPTED_LOGS_PER_TX
},
    hash::silo_note_hash,
    utils::arrays::{array_merge, assert_sorted_array, assert_deduped_array, check_permutation}
};

struct CombineHints {
    sorted_note_hashes: [ScopedNoteHash; MAX_NOTE_HASHES_PER_TX],
    sorted_note_hashes_indexes: [u32; MAX_NOTE_HASHES_PER_TX],
    sorted_note_encrypted_logs_hashes: [LogHash; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_note_encrypted_logs_hashes_indexes: [u32; MAX_NOTE_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_logs_hashes: [ScopedLogHash; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_encrypted_logs_hashes_indexes: [u32; MAX_ENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_logs_hashes: [ScopedLogHash; MAX_UNENCRYPTED_LOGS_PER_TX],
    sorted_unencrypted_logs_hashes_indexes: [u32; MAX_UNENCRYPTED_LOGS_PER_TX],
    // the public data update requests are sorted by their leaf index AND counter
    sorted_public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    sorted_public_data_update_requests_indexes: [u32; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    // THEN deduplicated based on their leaf slot
    deduped_public_data_update_requests: [PublicDataUpdateRequest; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
    deduped_public_data_update_requests_runs: [u32; MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX],
}

fn asc_sort_by_counters<T>(a: T, b: T) -> bool where T: Ordered {
    a.counter() <= b.counter()
}

pub fn combine_data(
    non_revertible: PublicAccumulatedData,
    revertible: PublicAccumulatedData,
    combine_hints: CombineHints
) -> CombinedAccumulatedData {
    let merged_note_hashes = array_merge(non_revertible.note_hashes, revertible.note_hashes);
    assert_sorted_array(
        merged_note_hashes,
        combine_hints.sorted_note_hashes,
        combine_hints.sorted_note_hashes_indexes,
        asc_sort_by_counters
    );

    let mut siloed_note_hashes = [0; MAX_NOTE_HASHES_PER_TX];
    let sorted_note_hashes = combine_hints.sorted_note_hashes;
    let tx_hash = non_revertible.nullifiers[0].value;
    for i in 0..sorted_note_hashes.len() {
        let note_hash = sorted_note_hashes[i];
        siloed_note_hashes[i] = if note_hash.counter() == 0 {
            // If counter is zero, the note hash was emitted from private and has been siloed in private_kernel_tail_to_public.
            note_hash.value()
        } else {
            silo_note_hash(note_hash, tx_hash, i)
        };
    }

    let merged_public_data_update_requests = array_merge(
        non_revertible.public_data_update_requests,
        revertible.public_data_update_requests
    );

    // Just check a permutation here...
    check_permutation(
        merged_public_data_update_requests,
        combine_hints.sorted_public_data_update_requests,
        combine_hints.sorted_public_data_update_requests_indexes
    );
    // ...because the ordering checks are done here.
    assert_deduped_array(
        combine_hints.sorted_public_data_update_requests,
        combine_hints.deduped_public_data_update_requests,
        combine_hints.deduped_public_data_update_requests_runs
    );

    let merged_note_encrypted_logs_hashes = array_merge(
        non_revertible.note_encrypted_logs_hashes,
        revertible.note_encrypted_logs_hashes
    );
    assert_sorted_array(
        merged_note_encrypted_logs_hashes,
        combine_hints.sorted_note_encrypted_logs_hashes,
        combine_hints.sorted_note_encrypted_logs_hashes_indexes,
        asc_sort_by_counters
    );

    let merged_encrypted_logs_hashes = array_merge(
        non_revertible.encrypted_logs_hashes,
        revertible.encrypted_logs_hashes
    );
    assert_sorted_array(
        merged_encrypted_logs_hashes,
        combine_hints.sorted_encrypted_logs_hashes,
        combine_hints.sorted_encrypted_logs_hashes_indexes,
        asc_sort_by_counters
    );

    let merged_unencrypted_logs_hashes = array_merge(
        non_revertible.unencrypted_logs_hashes,
        revertible.unencrypted_logs_hashes
    );
    assert_sorted_array(
        merged_unencrypted_logs_hashes,
        combine_hints.sorted_unencrypted_logs_hashes,
        combine_hints.sorted_unencrypted_logs_hashes_indexes,
        asc_sort_by_counters
    );

    let note_encrypted_log_preimages_length = non_revertible.note_encrypted_logs_hashes.fold(0, |a, b: LogHash| a + b.length)
        + revertible.note_encrypted_logs_hashes.fold(0, |a, b: LogHash| a + b.length);
    let encrypted_log_preimages_length = non_revertible.encrypted_logs_hashes.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length)
        + revertible.encrypted_logs_hashes.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length);
    let unencrypted_log_preimages_length = non_revertible.unencrypted_logs_hashes.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length)
        + revertible.unencrypted_logs_hashes.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length);
    CombinedAccumulatedData {
        note_hashes: siloed_note_hashes,
        nullifiers: array_merge(non_revertible.nullifiers, revertible.nullifiers).map(|n: Nullifier| n.value),
        l2_to_l1_msgs: array_merge(non_revertible.l2_to_l1_msgs, revertible.l2_to_l1_msgs),
        note_encrypted_logs_hashes: combine_hints.sorted_note_encrypted_logs_hashes,
        encrypted_logs_hashes: combine_hints.sorted_encrypted_logs_hashes,
        unencrypted_logs_hashes: combine_hints.sorted_unencrypted_logs_hashes,
        note_encrypted_log_preimages_length,
        encrypted_log_preimages_length,
        unencrypted_log_preimages_length,
        public_data_update_requests: combine_hints.deduped_public_data_update_requests,
        gas_used: revertible.gas_used + non_revertible.gas_used
    }
}
