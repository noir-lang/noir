use crate::components::tail_to_public_output_composer::meter_gas_used::{meter_gas_used_non_revertible, meter_gas_used_revertible};
use dep::types::{
    abis::gas::Gas,
    constants::{
    DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, FIXED_AVM_STARTUP_L2_GAS, L2_GAS_PER_NOTE_HASH,
    L2_GAS_PER_NULLIFIER, L2_GAS_PER_LOG_BYTE
},
    tests::fixture_builder::FixtureBuilder
};

#[test]
fn meter_gas_used_non_revertible_empty_succeeds() {
    let builder = FixtureBuilder::new();
    let data = builder.to_public_accumulated_data_builder();
    let gas = meter_gas_used_non_revertible(data);
    assert_eq(gas, Gas::tx_overhead());
}

#[test]
fn meter_gas_used_non_revertible_everything_succeeds() {
    let mut builder = FixtureBuilder::new();

    builder.append_note_hashes(4);
    builder.append_nullifiers(3);
    builder.append_l2_to_l1_msgs(1);
    builder.add_note_encrypted_log_hash(1001, 12, 0);
    builder.add_note_encrypted_log_hash(1002, 8, 0);
    builder.add_note_encrypted_log_hash(1003, 20, 0);
    builder.add_encrypted_log_hash(2001, 2);
    builder.add_encrypted_log_hash(2002, 6);
    builder.add_unencrypted_log_hash(3001, 51);
    builder.append_public_call_requests(2);
    builder.end_setup();

    let data = builder.to_public_accumulated_data_builder();
    let gas = meter_gas_used_non_revertible(data);

    let total_num_side_effects = 4 + 3 + 1;
    let total_log_length = 12 + 8 + 20 // note_encrypted_log_hash
        + 2 + 6 // encrypted_log_hash
        + 51; // unencrypted_log_hash
    let computed_da_gas = (total_num_side_effects * DA_BYTES_PER_FIELD + total_log_length) * DA_GAS_PER_BYTE;
    let computed_l2_gas = 4 * L2_GAS_PER_NOTE_HASH
        + 3 * L2_GAS_PER_NULLIFIER
        + total_log_length * L2_GAS_PER_LOG_BYTE
        + 2 * FIXED_AVM_STARTUP_L2_GAS;

    assert_eq(gas, Gas::new(computed_da_gas, computed_l2_gas) + Gas::tx_overhead());
}

#[test]
fn meter_gas_used_revertible_empty_succeeds() {
    let builder = FixtureBuilder::new();
    let data = builder.to_public_accumulated_data_builder();
    let teardown_gas = Gas::new(42, 17);
    let gas = meter_gas_used_revertible(data, teardown_gas);
    assert_eq(gas, teardown_gas);
}

#[test]
fn meter_gas_used_revertible_everything_succeeds() {
    let mut builder = FixtureBuilder::new();

    builder.append_note_hashes(4);
    builder.append_nullifiers(3);
    builder.append_l2_to_l1_msgs(1);
    builder.add_note_encrypted_log_hash(1001, 12, 0);
    builder.add_note_encrypted_log_hash(1002, 8, 0);
    builder.add_note_encrypted_log_hash(1003, 20, 0);
    builder.add_encrypted_log_hash(2001, 2);
    builder.add_encrypted_log_hash(2002, 6);
    builder.add_unencrypted_log_hash(3001, 51);
    builder.append_public_call_requests(2);
    builder.end_setup();

    let data = builder.to_public_accumulated_data_builder();
    let teardown_gas = Gas::new(42, 17);
    let gas = meter_gas_used_revertible(data, teardown_gas);

    let total_num_side_effects = 4 + 3 + 1;
    let total_log_length = 12 + 8 + 20 // note_encrypted_log_hash
        + 2 + 6 // encrypted_log_hash
        + 51; // unencrypted_log_hash
    let computed_da_gas = (total_num_side_effects * DA_BYTES_PER_FIELD + total_log_length) * DA_GAS_PER_BYTE;
    let computed_l2_gas = 4 * L2_GAS_PER_NOTE_HASH
        + 3 * L2_GAS_PER_NULLIFIER
        + total_log_length * L2_GAS_PER_LOG_BYTE
        + 2 * FIXED_AVM_STARTUP_L2_GAS;

    assert_eq(gas, Gas::new(computed_da_gas, computed_l2_gas) + teardown_gas);
}
