use dep::types::{
    abis::{
    accumulated_data::{public_accumulated_data_builder::PublicAccumulatedDataBuilder}, gas::Gas,
    log_hash::LogHash
},
    constants::{DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE}
};

fn meter_gas_used(data: PublicAccumulatedDataBuilder) -> Gas {
    let mut metered_bytes = 0;
    metered_bytes += data.note_hashes.len() * DA_BYTES_PER_FIELD;
    metered_bytes += data.nullifiers.len() * DA_BYTES_PER_FIELD;
    metered_bytes += data.l2_to_l1_msgs.len() * DA_BYTES_PER_FIELD;

    let note_encrypted_log_preimages_length = data.note_encrypted_logs_hashes.storage.fold(0, |len, l: LogHash| len + l.length);
    metered_bytes += note_encrypted_log_preimages_length as u32;

    let encrypted_log_preimages_length = data.encrypted_logs_hashes.storage.fold(0, |len, l: LogHash| len + l.length);
    metered_bytes += encrypted_log_preimages_length as u32;

    let unencrypted_log_preimages_length = data.unencrypted_logs_hashes.storage.fold(0, |len, l: LogHash| len + l.length);
    metered_bytes += unencrypted_log_preimages_length as u32;

    // TODO(gas): add AVM_STARTUP_L2_GAS

    Gas::new(metered_bytes * DA_GAS_PER_BYTE, 0)
}

pub fn meter_gas_used_non_revertible(data: PublicAccumulatedDataBuilder) -> Gas {
    meter_gas_used(data) + Gas::tx_overhead()
}

pub fn meter_gas_used_revertible(data: PublicAccumulatedDataBuilder, teardown_gas: Gas) -> Gas {
    meter_gas_used(data) + Gas::new(teardown_gas.da_gas, teardown_gas.l2_gas)
}
