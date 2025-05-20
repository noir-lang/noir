use crate::components::tail_to_public_output_composer::meter_gas_used;
use dep::types::{
    abis::{gas::Gas, public_call_request::PublicCallRequest},
    constants::{
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, FIXED_AVM_STARTUP_L2_GAS,
        L2_GAS_PER_CONTRACT_CLASS_LOG, L2_GAS_PER_L2_TO_L1_MSG, L2_GAS_PER_NOTE_HASH,
        L2_GAS_PER_NULLIFIER, L2_GAS_PER_PRIVATE_LOG,
    },
    tests::fixture_builder::FixtureBuilder,
    traits::Empty,
};

#[test]
fn meter_gas_used_empty_succeeds() {
    let builder = FixtureBuilder::new();
    let empty_data = builder.to_private_to_public_accumulated_data();
    let teardown_call_request = PublicCallRequest::empty();
    let teardown_gas_limits = Gas::new(1, 2);
    let gas = meter_gas_used(
        empty_data,
        empty_data,
        teardown_call_request,
        teardown_gas_limits,
    );
    assert_eq(gas, Gas::tx_overhead());
}

#[test]
fn meter_gas_used_with_teardown_call_succeeds() {
    let builder = FixtureBuilder::new();
    let empty_data = builder.to_private_to_public_accumulated_data();

    let mut teardown_call_request = PublicCallRequest::empty();
    teardown_call_request.contract_address = builder.contract_address;

    let teardown_gas_limits = Gas::new(1, 2);

    let gas = meter_gas_used(
        empty_data,
        empty_data,
        teardown_call_request,
        teardown_gas_limits,
    );
    assert_eq(gas, Gas::tx_overhead() + teardown_gas_limits);
}

#[test]
fn meter_gas_used_everything_succeeds() {
    // Non-revertible data.
    let mut non_revertible_builder = FixtureBuilder::new();

    let num_note_hashes_non_revertible = 3;
    non_revertible_builder.append_note_hashes(num_note_hashes_non_revertible);

    let num_nullifiers_non_revertible = 1;
    non_revertible_builder.append_nullifiers(num_nullifiers_non_revertible);

    let num_l2_to_l1_msgs_non_revertible = 0;
    non_revertible_builder.append_l2_to_l1_msgs(num_l2_to_l1_msgs_non_revertible);

    let private_log_lengths_non_revertible = [9, 6, 3];
    non_revertible_builder.append_private_logs_with_lengths(private_log_lengths_non_revertible);

    let contract_class_log_length_non_revertible = 51;
    non_revertible_builder.add_contract_class_log_hash(
        3001,
        contract_class_log_length_non_revertible,
    );

    let num_public_calls_non_revertible = 1;
    non_revertible_builder.append_public_call_requests(num_public_calls_non_revertible);

    // Revertible data.
    let mut revertible_builder = FixtureBuilder::new();

    let num_note_hashes_revertible = 1;
    revertible_builder.append_note_hashes(num_note_hashes_revertible);

    let num_nullifiers_revertible = 2;
    revertible_builder.append_nullifiers(num_nullifiers_revertible);

    let num_l2_to_l1_msgs_revertible = 1;
    revertible_builder.append_l2_to_l1_msgs(num_l2_to_l1_msgs_revertible);

    let private_log_lengths_revertible = [7, 11];
    revertible_builder.append_private_logs_with_lengths(private_log_lengths_revertible);

    let contract_class_log_length_revertible = 0;

    let num_public_calls_revertible = 1;
    revertible_builder.append_public_call_requests(num_public_calls_revertible);

    // Compute gas.
    let non_revertible_data = non_revertible_builder.to_private_to_public_accumulated_data();
    let revertible_data = revertible_builder.to_private_to_public_accumulated_data();
    let mut teardown_call_request = PublicCallRequest::empty();
    teardown_call_request.contract_address = non_revertible_builder.contract_address;
    let teardown_gas_limits = Gas::new(1, 2);

    let gas = meter_gas_used(
        non_revertible_data,
        revertible_data,
        teardown_call_request,
        teardown_gas_limits,
    );

    // Compute gas manually.
    let num_note_hashes = num_note_hashes_non_revertible + num_note_hashes_revertible;
    let num_nullifiers = num_nullifiers_non_revertible + num_nullifiers_revertible;
    let num_l2_to_l1_msgs = num_l2_to_l1_msgs_non_revertible + num_l2_to_l1_msgs_revertible;
    let num_public_calls = num_public_calls_non_revertible + num_public_calls_revertible;

    let num_private_logs =
        private_log_lengths_non_revertible.len() + private_log_lengths_revertible.len();
    let num_private_log_fields = private_log_lengths_non_revertible.fold(0, |acc, len| acc + len)
        + private_log_lengths_revertible.fold(0, |acc, len| acc + len);

    let num_cc_logs = 1;
    let num_cc_log_fields =
        contract_class_log_length_non_revertible + contract_class_log_length_revertible;

    let num_da_fields = num_note_hashes
        + num_nullifiers
        + num_l2_to_l1_msgs
        + num_private_log_fields
        + num_private_logs // One extra field per private log for its length.
        + num_cc_log_fields
        + (num_cc_logs * 2) // Two extra fields per contract class log: length and contract address.
    ;
    let computed_da_gas = num_da_fields * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;

    let computed_l2_gas = num_note_hashes * L2_GAS_PER_NOTE_HASH
        + num_nullifiers * L2_GAS_PER_NULLIFIER
        + num_l2_to_l1_msgs * L2_GAS_PER_L2_TO_L1_MSG
        + num_private_logs * L2_GAS_PER_PRIVATE_LOG
        + num_cc_logs * L2_GAS_PER_CONTRACT_CLASS_LOG
        + num_public_calls * FIXED_AVM_STARTUP_L2_GAS;

    assert_eq(
        gas,
        Gas::new(computed_da_gas, computed_l2_gas) + Gas::tx_overhead() + teardown_gas_limits,
    );
}
