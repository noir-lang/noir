use crate::components::tail_to_public_output_composer::split_to_public;
use dep::types::tests::{fixture_builder::FixtureBuilder, utils::assert_array_eq};

#[test]
fn split_to_public_succeeds() {
    let mut builder = FixtureBuilder::new();

    // Non-revertibles.
    builder.append_note_hashes(2);
    builder.append_nullifiers(2);
    builder.append_l2_to_l1_msgs(1);
    builder.append_private_logs(2);
    builder.add_contract_class_log_hash(2, 200);
    builder.append_public_call_requests(1);
    builder.end_setup();
    // Revertibles.
    builder.append_note_hashes(3);
    builder.append_nullifiers(1);
    builder.append_l2_to_l1_msgs(1);
    builder.append_private_logs(2);
    builder.append_public_call_requests(2);

    let combined_data = builder.to_private_to_public_accumulated_data();
    // Safety: This is a test.
    let (non_revertible, revertible) = unsafe {
        split_to_public(
            builder.to_private_accumulated_data_builder(),
            builder.min_revertible_side_effect_counter,
        )
    };

    // note_hashes
    let expected = combined_data.note_hashes;
    assert_array_eq(non_revertible.note_hashes, [expected[0], expected[1]]);
    assert_array_eq(
        revertible.note_hashes,
        [expected[2], expected[3], expected[4]],
    );

    // nullifiers
    let expected = combined_data.nullifiers;
    assert_array_eq(non_revertible.nullifiers, [expected[0], expected[1]]);
    assert_array_eq(revertible.nullifiers, [expected[2]]);

    // l2_to_l1_msgs
    let expected = combined_data.l2_to_l1_msgs;
    assert_array_eq(non_revertible.l2_to_l1_msgs, [expected[0]]);
    assert_array_eq(revertible.l2_to_l1_msgs, [expected[1]]);

    // private_logs
    let expected = combined_data.private_logs;
    assert_array_eq(non_revertible.private_logs, [expected[0], expected[1]]);
    assert_array_eq(revertible.private_logs, [expected[2], expected[3]]);

    // contract_class_logs_hashes
    let expected = combined_data.contract_class_logs_hashes;
    assert_array_eq(non_revertible.contract_class_logs_hashes, [expected[0]]);
    assert_array_eq(revertible.contract_class_logs_hashes, []);

    // public_call_requests
    let call_requests = combined_data.public_call_requests;
    assert_array_eq(non_revertible.public_call_requests, [call_requests[0]]);
    assert_array_eq(
        revertible.public_call_requests,
        [call_requests[1], call_requests[2]],
    );
}
