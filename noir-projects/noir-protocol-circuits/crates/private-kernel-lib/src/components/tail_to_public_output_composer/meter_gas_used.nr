use dep::types::{
    abis::{
    accumulated_data::{public_accumulated_data_builder::PublicAccumulatedDataBuilder}, gas::Gas,
    log_hash::LogHash
},
    constants::{
    DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, FIXED_AVM_STARTUP_L2_GAS, L2_GAS_PER_NOTE_HASH,
    L2_GAS_PER_NULLIFIER, L2_GAS_PER_LOG_BYTE
}
};

fn meter_gas_used(data: PublicAccumulatedDataBuilder) -> Gas {
    let mut metered_da_bytes = 0;
    let mut metered_l2_gas = 0;

    metered_da_bytes += data.note_hashes.len() * DA_BYTES_PER_FIELD;
    metered_l2_gas += data.note_hashes.len() * L2_GAS_PER_NOTE_HASH;

    metered_da_bytes += data.nullifiers.len() * DA_BYTES_PER_FIELD;
    metered_l2_gas += data.nullifiers.len() * L2_GAS_PER_NULLIFIER;

    metered_da_bytes += data.l2_to_l1_msgs.len() * DA_BYTES_PER_FIELD;

    let note_encrypted_log_preimages_length = data.note_encrypted_logs_hashes.storage.fold(0, |len, l: LogHash| len + l.length);
    metered_da_bytes += note_encrypted_log_preimages_length as u32;
    metered_l2_gas += note_encrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    let encrypted_log_preimages_length = data.encrypted_logs_hashes.storage.fold(0, |len, l: LogHash| len + l.length);
    metered_da_bytes += encrypted_log_preimages_length as u32;
    metered_l2_gas += encrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    let unencrypted_log_preimages_length = data.unencrypted_logs_hashes.storage.fold(0, |len, l: LogHash| len + l.length);
    metered_da_bytes += unencrypted_log_preimages_length as u32;
    metered_l2_gas += unencrypted_log_preimages_length as u32 * L2_GAS_PER_LOG_BYTE;

    metered_l2_gas += data.public_call_stack.len() * FIXED_AVM_STARTUP_L2_GAS;

    Gas::new(metered_da_bytes * DA_GAS_PER_BYTE, metered_l2_gas)
}

pub fn meter_gas_used_non_revertible(data: PublicAccumulatedDataBuilder) -> Gas {
    meter_gas_used(data) + Gas::tx_overhead()
}

pub fn meter_gas_used_revertible(data: PublicAccumulatedDataBuilder, teardown_gas: Gas) -> Gas {
    meter_gas_used(data) + Gas::new(teardown_gas.da_gas, teardown_gas.l2_gas)
}
