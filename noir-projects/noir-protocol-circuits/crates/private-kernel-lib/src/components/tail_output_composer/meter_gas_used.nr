use dep::types::{
    abis::{
    accumulated_data::combined_accumulated_data::CombinedAccumulatedData, gas::Gas,
    gas_settings::GasSettings
},
    constants::{DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, L2_GAS_PER_NOTE_HASH, L2_GAS_PER_NULLIFIER, L2_GAS_PER_LOG_BYTE},
    utils::arrays::array_length
};

fn meter_gas_used(data: CombinedAccumulatedData, gas_settings: GasSettings) -> Gas {
    let mut metered_da_bytes = 0;
    let mut metered_l2_gas = 0;

    let num_note_hashes = array_length(data.note_hashes);
    metered_da_bytes += num_note_hashes * DA_BYTES_PER_FIELD;
    metered_l2_gas += num_note_hashes * L2_GAS_PER_NOTE_HASH;

    let num_nullifiers = array_length(data.nullifiers);
    metered_da_bytes += num_nullifiers * DA_BYTES_PER_FIELD;
    metered_l2_gas += num_nullifiers * L2_GAS_PER_NULLIFIER;

    let num_l2_to_l1_msgs = array_length(data.l2_to_l1_msgs);
    metered_da_bytes += num_l2_to_l1_msgs * DA_BYTES_PER_FIELD;

    metered_da_bytes += data.note_encrypted_log_preimages_length as u32;
    metered_l2_gas += data.note_encrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    metered_da_bytes += data.encrypted_log_preimages_length as u32;
    metered_l2_gas += data.encrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    metered_da_bytes += data.unencrypted_log_preimages_length as u32;
    metered_l2_gas += data.unencrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    let teardown_gas = gas_settings.teardown_gas_limits;
    Gas::new(metered_da_bytes * DA_GAS_PER_BYTE, metered_l2_gas) + Gas::tx_overhead() + teardown_gas
}
