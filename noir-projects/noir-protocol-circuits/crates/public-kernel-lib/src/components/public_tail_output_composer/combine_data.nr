use dep::types::{
    abis::{
    accumulated_data::{CombinedAccumulatedData, PublicAccumulatedData},
    log_hash::{LogHash, ScopedLogHash}, nullifier::Nullifier
},
    constants::MAX_NOTE_HASHES_PER_TX, hash::silo_note_hash,
    utils::arrays::{array_merge, dedupe_array, sort_by_counter_asc}
};

unconstrained pub fn combine_data(
    non_revertible: PublicAccumulatedData,
    revertible: PublicAccumulatedData
) -> CombinedAccumulatedData {
    let mut note_hashes = [0; MAX_NOTE_HASHES_PER_TX];
    let sorted_unsiloed_note_hashes = sort_by_counter_asc(array_merge(non_revertible.note_hashes, revertible.note_hashes));
    let tx_hash = non_revertible.nullifiers[0].value;
    for i in 0..sorted_unsiloed_note_hashes.len() {
        let note_hash = sorted_unsiloed_note_hashes[i];
        note_hashes[i] = if note_hash.counter() == 0 {
            // If counter is zero, the note hash is either empty or is emitted from private and has been siloed in private_kernel_tail_to_public.
            note_hash.value()
        } else {
            silo_note_hash(note_hash, tx_hash, i)
        };
    }

    let nullifiers = sort_by_counter_asc(array_merge(non_revertible.nullifiers, revertible.nullifiers)).map(|n: Nullifier| n.value);

    let l2_to_l1_msgs = sort_by_counter_asc(array_merge(non_revertible.l2_to_l1_msgs, revertible.l2_to_l1_msgs));

    let public_data_update_requests = dedupe_array(
        array_merge(
            non_revertible.public_data_update_requests,
            revertible.public_data_update_requests
        )
    );

    let note_encrypted_logs_hashes = sort_by_counter_asc(
        array_merge(
            non_revertible.note_encrypted_logs_hashes,
            revertible.note_encrypted_logs_hashes
        )
    );
    let note_encrypted_log_preimages_length = note_encrypted_logs_hashes.fold(0, |a, b: LogHash| a + b.length);

    let encrypted_logs_hashes = sort_by_counter_asc(
        array_merge(
            non_revertible.encrypted_logs_hashes,
            revertible.encrypted_logs_hashes
        )
    );
    let encrypted_log_preimages_length = encrypted_logs_hashes.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length);

    let unencrypted_logs_hashes = sort_by_counter_asc(
        array_merge(
            non_revertible.unencrypted_logs_hashes,
            revertible.unencrypted_logs_hashes
        )
    );
    let unencrypted_log_preimages_length = unencrypted_logs_hashes.fold(0, |a, b: ScopedLogHash| a + b.log_hash.length);

    CombinedAccumulatedData {
        note_hashes,
        nullifiers,
        l2_to_l1_msgs,
        note_encrypted_logs_hashes,
        encrypted_logs_hashes,
        unencrypted_logs_hashes,
        note_encrypted_log_preimages_length,
        encrypted_log_preimages_length,
        unencrypted_log_preimages_length,
        public_data_update_requests,
        gas_used: revertible.gas_used + non_revertible.gas_used
    }
}
