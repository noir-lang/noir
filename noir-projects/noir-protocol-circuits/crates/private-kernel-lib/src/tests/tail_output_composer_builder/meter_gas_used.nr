use crate::components::tail_output_composer::meter_gas_used::meter_gas_used;
use dep::types::{
    abis::gas::Gas,
    constants::{DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, L2_GAS_PER_NOTE_HASH, L2_GAS_PER_NULLIFIER, L2_GAS_PER_LOG_BYTE},
    tests::fixture_builder::FixtureBuilder
};

fn new_builder() -> FixtureBuilder {
    let mut builder = FixtureBuilder::new();
    builder.tx_context.gas_settings.teardown_gas_limits = Gas::new(12, 345);
    builder
}

#[test]
fn meter_gas_used_empty_succeeds() {
    let builder = new_builder();
    let data = builder.to_combined_accumulated_data();
    let gas_settings = builder.tx_context.gas_settings;
    let gas = meter_gas_used(data, gas_settings);
    assert_eq(gas, Gas::tx_overhead() + gas_settings.teardown_gas_limits);
}

#[test]
fn meter_gas_used_everything_succeeds() {
    let mut builder = new_builder();
    let mut metered_da_bytes = 0;
    let mut computed_l2_gas = 0;

    builder.append_note_hashes(4);
    metered_da_bytes += 4 * DA_BYTES_PER_FIELD;
    computed_l2_gas += 4 * L2_GAS_PER_NOTE_HASH;

    builder.append_nullifiers(3);
    metered_da_bytes += 3 * DA_BYTES_PER_FIELD;
    computed_l2_gas += 3 * L2_GAS_PER_NULLIFIER;

    builder.append_l2_to_l1_msgs(1);
    metered_da_bytes += 1 * DA_BYTES_PER_FIELD;

    builder.add_note_encrypted_log_hash(1001, 12, 0);
    metered_da_bytes += 12;
    computed_l2_gas += 12 * L2_GAS_PER_LOG_BYTE;

    builder.add_note_encrypted_log_hash(1002, 8, 0);
    metered_da_bytes += 8;
    computed_l2_gas += 8 * L2_GAS_PER_LOG_BYTE;

    builder.add_note_encrypted_log_hash(1003, 20, 0);
    metered_da_bytes += 20;
    computed_l2_gas += 20 * L2_GAS_PER_LOG_BYTE;

    builder.add_encrypted_log_hash(2001, 2);
    metered_da_bytes += 2;
    computed_l2_gas += 2 * L2_GAS_PER_LOG_BYTE;

    builder.add_encrypted_log_hash(2002, 6);
    metered_da_bytes += 6;
    computed_l2_gas += 6 * L2_GAS_PER_LOG_BYTE;

    builder.add_unencrypted_log_hash(3001, 51);
    metered_da_bytes += 51;
    computed_l2_gas += 51 * L2_GAS_PER_LOG_BYTE;

    let data = builder.to_combined_accumulated_data();
    let gas_settings = builder.tx_context.gas_settings;
    let gas = meter_gas_used(data, gas_settings);

    assert_eq(
        gas, Gas::new(metered_da_bytes * DA_GAS_PER_BYTE, computed_l2_gas)
        + Gas::tx_overhead()
        + gas_settings.teardown_gas_limits
    );
}
