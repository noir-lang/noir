use dep::types::{
    abis::{
        accumulated_data::private_to_rollup_accumulated_data::PrivateToRollupAccumulatedData,
        gas::Gas,
    },
    constants::{
        DA_BYTES_PER_FIELD, DA_GAS_PER_BYTE, L2_GAS_PER_CONTRACT_CLASS_LOG, L2_GAS_PER_L2_TO_L1_MSG,
        L2_GAS_PER_NOTE_HASH, L2_GAS_PER_NULLIFIER, L2_GAS_PER_PRIVATE_LOG,
    },
    utils::arrays::array_length,
};

pub fn meter_gas_used(data: PrivateToRollupAccumulatedData) -> Gas {
    let mut metered_da_fields = 0;
    let mut metered_l2_gas = 0;

    let num_note_hashes = array_length(data.note_hashes);
    metered_da_fields += num_note_hashes;
    metered_l2_gas += num_note_hashes * L2_GAS_PER_NOTE_HASH;

    let num_nullifiers = array_length(data.nullifiers);
    metered_da_fields += num_nullifiers;
    metered_l2_gas += num_nullifiers * L2_GAS_PER_NULLIFIER;

    let num_l2_to_l1_msgs = array_length(data.l2_to_l1_msgs);
    metered_da_fields += num_l2_to_l1_msgs;
    metered_l2_gas += num_l2_to_l1_msgs * L2_GAS_PER_L2_TO_L1_MSG;

    let num_private_logs = array_length(data.private_logs);
    metered_da_fields += data.private_logs.fold(0, |acc, l| acc + l.length);
    // Each private log emits its length as an additional field.
    // It is slightly more efficient to account for all extra fields at once, rather than adding one field per log individually.
    metered_da_fields += num_private_logs;
    metered_l2_gas += num_private_logs * L2_GAS_PER_PRIVATE_LOG;

    let num_contract_class_logs = array_length(data.contract_class_logs_hashes);
    metered_da_fields += data.contract_class_logs_hashes.fold(0, |acc, l| acc + l.inner.length);
    // Each contract class log emits its length and contract address as additional fields.
    // It is slightly more efficient to account for all extra fields at once, rather than adding one field per log individually.
    metered_da_fields += num_contract_class_logs * 2;
    metered_l2_gas += num_contract_class_logs * L2_GAS_PER_CONTRACT_CLASS_LOG;

    let metered_da_gas = metered_da_fields * DA_BYTES_PER_FIELD * DA_GAS_PER_BYTE;

    Gas::new(metered_da_gas, metered_l2_gas) + Gas::tx_overhead()
}
