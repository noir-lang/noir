use crate::tests::tail_to_public_output_composer_builder::TailToPublicOutputComposerBuilder;
use dep::types::{
    abis::gas::Gas,
    constants::{
    DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, FIXED_AVM_STARTUP_L2_GAS, L2_GAS_PER_NOTE_HASH,
    L2_GAS_PER_NULLIFIER, L2_GAS_PER_LOG_BYTE
},
    tests::utils::{assert_array_eq, swap_items}
};

#[test]
fn tail_to_public_output_composer_succeeds() {
    let mut builder = TailToPublicOutputComposerBuilder::new();

    let teardown_gas = Gas::new(789, 3254);
    builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = teardown_gas;

    // Non-revertibles.
    builder.previous_kernel.append_siloed_note_hashes(4);

    builder.previous_kernel.append_siloed_nullifiers(2);

    builder.previous_kernel.append_l2_to_l1_msgs(1);

    builder.previous_kernel.add_note_encrypted_log_hash(1001, 12, 0);
    builder.previous_kernel.add_note_encrypted_log_hash(1002, 8, 0);

    builder.previous_kernel.add_masked_encrypted_log_hash(2001, 2);

    builder.previous_kernel.add_unencrypted_log_hash(3001, 51);
    builder.previous_kernel.add_unencrypted_log_hash(3002, 9);

    builder.previous_kernel.append_public_call_requests(2);

    builder.previous_kernel.end_setup();

    // Revertibles.
    builder.previous_kernel.append_siloed_note_hashes(2);

    builder.previous_kernel.append_siloed_nullifiers(1);

    builder.previous_kernel.append_l2_to_l1_msgs(1);

    builder.previous_kernel.add_note_encrypted_log_hash(1003, 20, 0);

    builder.previous_kernel.add_masked_encrypted_log_hash(2002, 6);
    builder.previous_kernel.add_masked_encrypted_log_hash(2003, 24);

    builder.previous_kernel.add_unencrypted_log_hash(3002, 4);

    builder.previous_kernel.append_public_call_requests(3);

    // Get ordered items before shuffling for verifying with the output later.
    let data = builder.previous_kernel.to_exposed_public_accumulated_data();

    // Shuffle ordered items.
    swap_items(&mut builder.previous_kernel.l2_to_l1_msgs, 0, 1);
    swap_items(&mut builder.previous_kernel.unencrypted_logs_hashes, 0, 2);
    swap_items(&mut builder.previous_kernel.public_call_requests, 1, 2);

    // Output.
    let output = builder.finish();

    // note_hashes
    let note_hashes = data.note_hashes;
    assert_array_eq(
        output.end_non_revertible.note_hashes,
        [note_hashes[0], note_hashes[1], note_hashes[2], note_hashes[3]]
    );
    assert_array_eq(output.end.note_hashes, [note_hashes[4], note_hashes[5]]);

    // nullifiers
    let nullifiers = data.nullifiers;
    assert_array_eq(
        output.end_non_revertible.nullifiers,
        [nullifiers[0], nullifiers[1], nullifiers[2]]
    );
    assert_array_eq(output.end.nullifiers, [nullifiers[3]]);

    // l2_to_l1_msgs
    let msgs = data.l2_to_l1_msgs;
    assert_array_eq(output.end_non_revertible.l2_to_l1_msgs, [msgs[0]]);
    assert_array_eq(output.end.l2_to_l1_msgs, [msgs[1]]);

    // note_encrypted_logs_hashes
    let log_hashes = data.note_encrypted_logs_hashes;
    assert_array_eq(
        output.end_non_revertible.note_encrypted_logs_hashes,
        [log_hashes[0], log_hashes[1]]
    );
    assert_array_eq(output.end.note_encrypted_logs_hashes, [log_hashes[2]]);

    // encrypted_logs_hashes
    let log_hashes = data.encrypted_logs_hashes;
    assert_array_eq(
        output.end_non_revertible.encrypted_logs_hashes,
        [log_hashes[0]]
    );
    assert_array_eq(
        output.end.encrypted_logs_hashes,
        [log_hashes[1], log_hashes[2]]
    );

    // unencrypted_logs_hashes
    let log_hashes = data.unencrypted_logs_hashes;
    assert_array_eq(
        output.end_non_revertible.unencrypted_logs_hashes,
        [log_hashes[0], log_hashes[1]]
    );
    assert_array_eq(output.end.unencrypted_logs_hashes, [log_hashes[2]]);

    // public_call_stack
    let call_requests = data.public_call_stack;
    assert_array_eq(
        output.end_non_revertible.public_call_stack,
        [call_requests[1], call_requests[0]]
    );
    assert_array_eq(
        output.end.public_call_stack,
        [call_requests[4], call_requests[3], call_requests[2]]
    );

    // Gas: non-revertible
    let total_num_side_effects = 4 + 3 + 1;
    let total_log_length = 12 + 8 // note_encrypted_log_hash
        + 2 // encrypted_log_hash
        + 51 + 9; // unencrypted_log_hash
    let computed_da_gas = (total_num_side_effects * DA_BYTES_PER_FIELD + total_log_length) * DA_GAS_PER_BYTE;
    let computed_l2_gas = 4 * L2_GAS_PER_NOTE_HASH
        + 3 * L2_GAS_PER_NULLIFIER
        + total_log_length * L2_GAS_PER_LOG_BYTE
        + 2 * FIXED_AVM_STARTUP_L2_GAS;
    assert_eq(
        output.end_non_revertible.gas_used, Gas::new(computed_da_gas, computed_l2_gas) + Gas::tx_overhead()
    );

    // Gas: revertible
    let total_num_side_effects = 2 + 1 + 1;
    let total_log_length = 20 // note_encrypted_log_hash
        + 6 + 24 // encrypted_log_hash
        + 4; // unencrypted_log_hash
    let computed_da_gas = (total_num_side_effects * DA_BYTES_PER_FIELD + total_log_length) * DA_GAS_PER_BYTE;
    let computed_l2_gas = 2 * L2_GAS_PER_NOTE_HASH
        + 1 * L2_GAS_PER_NULLIFIER
        + total_log_length * L2_GAS_PER_LOG_BYTE
        + 3 * FIXED_AVM_STARTUP_L2_GAS;
    assert_eq(output.end.gas_used, Gas::new(computed_da_gas, computed_l2_gas) + teardown_gas);
}
