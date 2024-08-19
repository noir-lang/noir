use dep::types::{
    abis::{
    accumulated_data::PublicAccumulatedDataBuilder, log_hash::LogHash, nullifier::Nullifier,
    public_circuit_public_inputs::PublicCircuitPublicInputs,
    public_data_update_request::PublicDataUpdateRequest
},
    hash::compute_siloed_nullifier
};

pub fn propagate_accumulated_data(
    data: &mut PublicAccumulatedDataBuilder,
    public_call: PublicCircuitPublicInputs
) -> PublicAccumulatedDataBuilder {
    propagate_note_hashes(data, public_call);
    propagate_nullifiers(data, public_call);
    propagate_l2_to_l1_messages(data, public_call);
    propagate_unencrypted_logs(data, public_call);
    propagate_public_data_writes(data, public_call);
    *data
}

fn propagate_note_hashes(data: &mut PublicAccumulatedDataBuilder, public_call: PublicCircuitPublicInputs) {
    let storage_contract_address = public_call.call_context.storage_contract_address;
    let note_hashes = public_call.note_hashes;
    for i in 0..note_hashes.len() {
        let note_hash = note_hashes[i];
        if note_hash.counter != 0 {
            data.note_hashes.push(note_hash.scope(storage_contract_address));
        }
    }
}

fn propagate_nullifiers(data: &mut PublicAccumulatedDataBuilder, public_call: PublicCircuitPublicInputs) {
    let storage_contract_address = public_call.call_context.storage_contract_address;
    let nullifiers = public_call.nullifiers;
    for i in 0..nullifiers.len() {
        let nullifier = nullifiers[i];
        if nullifier.counter != 0 {
            let siloed_value = compute_siloed_nullifier(storage_contract_address, nullifier.value);
            data.nullifiers.push(
                Nullifier { value: siloed_value, counter: nullifier.counter, note_hash: nullifier.note_hash }
            );
        }
    }
}

fn propagate_l2_to_l1_messages(data: &mut PublicAccumulatedDataBuilder, public_call: PublicCircuitPublicInputs) {
    let storage_contract_address = public_call.call_context.storage_contract_address;
    let msgs = public_call.l2_to_l1_msgs;
    for i in 0..msgs.len() {
        let msg = msgs[i];
        if msg.counter != 0 {
            data.l2_to_l1_msgs.push(msg.scope(storage_contract_address));
        }
    }
}

fn propagate_unencrypted_logs(data: &mut PublicAccumulatedDataBuilder, public_call: PublicCircuitPublicInputs) {
    let storage_contract_address = public_call.call_context.storage_contract_address;
    let logs = public_call.unencrypted_logs_hashes;
    for i in 0..logs.len() {
        let log = logs[i];
        if log.counter != 0 {
            data.unencrypted_logs_hashes.push(log.scope(storage_contract_address));
        }
    }
}

fn propagate_public_data_writes(data: &mut PublicAccumulatedDataBuilder, public_call: PublicCircuitPublicInputs) {
    let storage_contract_address = public_call.call_context.storage_contract_address;
    let writes = public_call.contract_storage_update_requests;
    for i in 0..writes.len() {
        let write = writes[i];
        if write.counter != 0 {
            data.public_data_update_requests.push(
                PublicDataUpdateRequest::from_contract_storage_update_request(storage_contract_address, write)
            );
        }
    }
}
