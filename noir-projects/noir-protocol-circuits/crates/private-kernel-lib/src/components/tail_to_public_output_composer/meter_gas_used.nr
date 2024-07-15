use dep::types::{
    abis::{
    accumulated_data::{public_accumulated_data_builder::PublicAccumulatedData}, gas::Gas,
    log_hash::{LogHash, ScopedLogHash}
},
    constants::{
    DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, FIXED_AVM_STARTUP_L2_GAS, L2_GAS_PER_NOTE_HASH,
    L2_GAS_PER_NULLIFIER, L2_GAS_PER_LOG_BYTE
},
    utils::arrays::array_length
};

fn meter_gas_used(data: PublicAccumulatedData) -> Gas {
    let mut metered_da_bytes = 0;
    let mut metered_l2_gas = 0;

    let num_note_hashes = array_length(data.note_hashes);
    metered_da_bytes += num_note_hashes * DA_BYTES_PER_FIELD;
    metered_l2_gas += num_note_hashes * L2_GAS_PER_NOTE_HASH;

    let num_nullifiers = array_length(data.nullifiers);
    metered_da_bytes += num_nullifiers * DA_BYTES_PER_FIELD;
    metered_l2_gas += num_nullifiers * L2_GAS_PER_NULLIFIER;

    metered_da_bytes += array_length(data.l2_to_l1_msgs) * DA_BYTES_PER_FIELD;

    let note_encrypted_log_preimages_length = data.note_encrypted_logs_hashes.fold(0, |len, l: LogHash| len + l.length);
    metered_da_bytes += note_encrypted_log_preimages_length as u32;
    metered_l2_gas += note_encrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    let encrypted_log_preimages_length = data.encrypted_logs_hashes.fold(0, |len, l: LogHash| len + l.length);
    metered_da_bytes += encrypted_log_preimages_length as u32;
    metered_l2_gas += encrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    let unencrypted_log_preimages_length = data.unencrypted_logs_hashes.fold(0, |len, l: ScopedLogHash| len + l.log_hash.length);
    metered_da_bytes += unencrypted_log_preimages_length as u32;
    metered_l2_gas += unencrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    metered_l2_gas += array_length(data.public_call_stack) * FIXED_AVM_STARTUP_L2_GAS;

    Gas::new(metered_da_bytes * DA_GAS_PER_BYTE, metered_l2_gas)
}

pub fn meter_gas_used_non_revertible(data: PublicAccumulatedData) -> Gas {
    meter_gas_used(data) + Gas::tx_overhead()
}

pub fn meter_gas_used_revertible(data: PublicAccumulatedData, teardown_gas: Gas) -> Gas {
    meter_gas_used(data) + Gas::new(teardown_gas.da_gas, teardown_gas.l2_gas)
}
