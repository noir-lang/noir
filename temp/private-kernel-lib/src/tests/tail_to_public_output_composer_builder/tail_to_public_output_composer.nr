use crate::tests::tail_to_public_output_composer_builder::TailToPublicOutputComposerBuilder;
use dep::types::{
    abis::gas::Gas,
    constants::{
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, FIXED_AVM_STARTUP_L2_GAS,
        L2_GAS_PER_CONTRACT_CLASS_LOG, L2_GAS_PER_L2_TO_L1_MSG, L2_GAS_PER_NOTE_HASH,
        L2_GAS_PER_NULLIFIER, L2_GAS_PER_PRIVATE_LOG,
    },
    tests::utils::{assert_array_eq, swap_items},
};

#[test]
fn tail_to_public_output_composer_succeeds() {
    let mut builder = TailToPublicOutputComposerBuilder::new();

    let teardown_gas_limits = Gas::new(789, 3254);
    builder.previous_kernel.tx_context.gas_settings.teardown_gas_limits = teardown_gas_limits;

    // Non-revertible.
    let num_note_hashes_non_revertible = 4;
    builder.previous_kernel.append_siloed_note_hashes(num_note_hashes_non_revertible);

    let mut num_nullifiers_non_revertible = 2;
    builder.previous_kernel.append_siloed_nullifiers(num_nullifiers_non_revertible);
    // The builder always add the protocol nullifier, so we need to add one more.
    num_nullifiers_non_revertible += 1;

    let num_l2_to_l1_msgs_non_revertible = 1;
    builder.previous_kernel.append_l2_to_l1_msgs(num_l2_to_l1_msgs_non_revertible);

    let private_log_lengths_non_revertible = [8, 11];
    builder.previous_kernel.append_private_logs_with_lengths(private_log_lengths_non_revertible);

    let contract_class_log_length_non_revertible = 9;
    builder.previous_kernel.add_contract_class_log_hash(
        3002,
        contract_class_log_length_non_revertible,
    );

    let num_public_calls_non_revertible = 2;
    builder.previous_kernel.append_public_call_requests(num_public_calls_non_revertible);

    builder.previous_kernel.end_setup();

    // Revertible.
    builder.previous_kernel.set_public_teardown_call_request();

    let num_note_hashes_revertible = 2;
    builder.previous_kernel.append_siloed_note_hashes(num_note_hashes_revertible);

    let num_nullifiers_revertible = 1;
    builder.previous_kernel.append_siloed_nullifiers(num_nullifiers_revertible);

    let num_l2_to_l1_msgs_revertible = 1;
    builder.previous_kernel.append_l2_to_l1_msgs(num_l2_to_l1_msgs_revertible);

    let private_log_lengths_revertible = [7];
    builder.previous_kernel.append_private_logs_with_lengths(private_log_lengths_revertible);

    let contract_class_log_length_revertible = 0;

    let num_public_calls_revertible = 3;
    builder.previous_kernel.append_public_call_requests(num_public_calls_revertible);

    // Get ordered items before shuffling for verifying with the output later.
    let data = builder.previous_kernel.to_private_to_public_accumulated_data();

    // Shuffle ordered items.
    swap_items(&mut builder.previous_kernel.l2_to_l1_msgs, 0, 1);
    swap_items(&mut builder.previous_kernel.private_logs, 1, 2);
    swap_items(&mut builder.previous_kernel.public_call_requests, 1, 3);

    // Output.
    let output = builder.finish();

    // note_hashes
    let note_hashes = data.note_hashes;
    assert_array_eq(
        output.non_revertible_accumulated_data.note_hashes,
        [note_hashes[0], note_hashes[1], note_hashes[2], note_hashes[3]],
    );
    assert_array_eq(
        output.revertible_accumulated_data.note_hashes,
        [note_hashes[4], note_hashes[5]],
    );

    // nullifiers
    let nullifiers = data.nullifiers;
    assert_array_eq(
        output.non_revertible_accumulated_data.nullifiers,
        [nullifiers[0], nullifiers[1], nullifiers[2]],
    );
    assert_array_eq(
        output.revertible_accumulated_data.nullifiers,
        [nullifiers[3]],
    );

    // l2_to_l1_msgs
    let msgs = data.l2_to_l1_msgs;
    assert_array_eq(
        output.non_revertible_accumulated_data.l2_to_l1_msgs,
        [msgs[0]],
    );
    assert_array_eq(output.revertible_accumulated_data.l2_to_l1_msgs, [msgs[1]]);

    // private_logs
    let private_logs = data.private_logs;
    assert_array_eq(
        output.non_revertible_accumulated_data.private_logs,
        [private_logs[0], private_logs[1]],
    );
    assert_array_eq(
        output.revertible_accumulated_data.private_logs,
        [private_logs[2]],
    );

    // contract_class_logs_hashes
    let log_hashes = data.contract_class_logs_hashes;
    assert_array_eq(
        output.non_revertible_accumulated_data.contract_class_logs_hashes,
        [log_hashes[0]],
    );

    // public_call_requests
    let public_call_requests = data.public_call_requests;
    assert_array_eq(
        output.non_revertible_accumulated_data.public_call_requests,
        [public_call_requests[1], public_call_requests[0]],
    );
    assert_array_eq(
        output.revertible_accumulated_data.public_call_requests,
        [public_call_requests[4], public_call_requests[3], public_call_requests[2]],
    );

    // Compute gas.
    let num_note_hashes = num_note_hashes_non_revertible + num_note_hashes_revertible;
    let num_nullifiers = num_nullifiers_non_revertible + num_nullifiers_revertible;
    let num_l2_to_l1_msgs = num_l2_to_l1_msgs_non_revertible + num_l2_to_l1_msgs_revertible;
    let num_public_calls = num_public_calls_non_revertible + num_public_calls_revertible;

    let num_private_logs =
        private_log_lengths_non_revertible.len() + private_log_lengths_revertible.len();
    let num_private_log_fields = private_log_lengths_non_revertible.fold(0, |acc, len| acc + len)
        + private_log_lengths_revertible.fold(0, |acc, len| acc + len);

    let num_contract_class_logs = 1;
    let num_contract_class_log_fields =
        contract_class_log_length_non_revertible + contract_class_log_length_revertible;

    let num_da_fields = num_note_hashes
        + num_nullifiers
        + num_l2_to_l1_msgs
        + num_private_log_fields
        + num_private_logs // One extra field per private log for its length.
        + num_contract_class_log_fields
        + (num_contract_class_logs * 2) // Two extra fields per contract class log: length and contract address.
    ;
    let computed_da_gas = num_da_fields * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;

    let computed_l2_gas = num_note_hashes * L2_GAS_PER_NOTE_HASH
        + num_nullifiers * L2_GAS_PER_NULLIFIER
        + num_l2_to_l1_msgs * L2_GAS_PER_L2_TO_L1_MSG
        + num_private_logs * L2_GAS_PER_PRIVATE_LOG
        + num_contract_class_logs * L2_GAS_PER_CONTRACT_CLASS_LOG
        + num_public_calls * FIXED_AVM_STARTUP_L2_GAS;

    assert_eq(
        output.gas_used,
        Gas::tx_overhead() + Gas::new(computed_da_gas, computed_l2_gas) + teardown_gas_limits,
    );
}
