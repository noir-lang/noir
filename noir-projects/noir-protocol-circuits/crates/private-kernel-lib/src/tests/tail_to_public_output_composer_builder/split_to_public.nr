use crate::components::tail_to_public_output_composer::split_to_public::split_to_public;
use dep::types::tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq};

#[test]
fn split_to_public_succeeds() {
    let mut builder = FixtureBuilder::new();

    // Non-revertibles.
    builder.append_note_hashes(2);
    builder.append_nullifiers(2);
    builder.append_l2_to_l1_msgs(1);
    builder.append_note_encrypted_log_hashes(3);
    builder.append_encrypted_log_hashes(2);
    builder.append_unencrypted_log_hashes(1);
    builder.append_public_call_requests(1);
    builder.end_setup();
    // Revertibles.
    builder.append_note_hashes(3);
    builder.append_nullifiers(1);
    builder.append_l2_to_l1_msgs(1);
    builder.append_note_encrypted_log_hashes(1);
    builder.append_encrypted_log_hashes(2);
    builder.append_unencrypted_log_hashes(1);
    builder.append_public_call_requests(2);

    let combined_data = builder.to_exposed_public_accumulated_data();
    let (non_revertible, revertible) = split_to_public(
        builder.to_private_accumulated_data_builder(),
        builder.min_revertible_side_effect_counter
    );

    // note_hashes
    let expected = combined_data.note_hashes;
    assert_array_eq(
        non_revertible.note_hashes.storage,
        [expected[0], expected[1]]
    );
    assert_array_eq(
        revertible.note_hashes.storage,
        [expected[2], expected[3], expected[4]]
    );

    // nullifiers
    let expected = combined_data.nullifiers;
    assert_array_eq(non_revertible.nullifiers.storage, [expected[0], expected[1]]);
    assert_array_eq(revertible.nullifiers.storage, [expected[2]]);

    // l2_to_l1_msgs
    let expected = combined_data.l2_to_l1_msgs;
    assert_array_eq(non_revertible.l2_to_l1_msgs.storage, [expected[0]]);
    assert_array_eq(revertible.l2_to_l1_msgs.storage, [expected[1]]);

    // note_encrypted_logs_hashes
    let expected = combined_data.note_encrypted_logs_hashes;
    assert_array_eq(
        non_revertible.note_encrypted_logs_hashes.storage,
        [expected[0], expected[1], expected[2]]
    );
    assert_array_eq(revertible.note_encrypted_logs_hashes.storage, [expected[3]]);

    // encrypted_logs_hashes
    let expected = combined_data.encrypted_logs_hashes;
    assert_array_eq(
        non_revertible.encrypted_logs_hashes.storage,
        [expected[0], expected[1]]
    );
    assert_array_eq(
        revertible.encrypted_logs_hashes.storage,
        [expected[2], expected[3]]
    );

    // unencrypted_logs_hashes
    let expected = combined_data.unencrypted_logs_hashes;
    assert_array_eq(non_revertible.unencrypted_logs_hashes.storage, [expected[0]]);
    assert_array_eq(revertible.unencrypted_logs_hashes.storage, [expected[1]]);

    // public_call_stack
    let expected = combined_data.public_call_stack;
    assert_array_eq(non_revertible.public_call_stack.storage, [expected[0]]);
    assert_array_eq(
        revertible.public_call_stack.storage,
        [expected[1], expected[2]]
    );
}

#[test(should_fail_with="min_revertible_side_effect_counter must not be 0")]
fn split_to_public_zero_counter_fails() {
    let mut builder = FixtureBuilder::new();

    builder.append_note_hashes(1);

    let _ = split_to_public(
        builder.to_private_accumulated_data_builder(),
        builder.min_revertible_side_effect_counter
    );
}
