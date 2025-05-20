use dep::types::{
    abis::{
        accumulated_data::{
            private_accumulated_data_builder::PrivateAccumulatedDataBuilder,
            private_to_public_accumulated_data::PrivateToPublicAccumulatedData,
            private_to_public_accumulated_data_builder::PrivateToPublicAccumulatedDataBuilder,
        },
        side_effect::{Ordered, OrderedValue},
    },
    traits::Empty,
};

pub unconstrained fn split_to_public(
    data: PrivateAccumulatedDataBuilder,
    min_revertible_side_effect_counter: u32,
) -> (PrivateToPublicAccumulatedData, PrivateToPublicAccumulatedData) {
    let mut non_revertible_builder = PrivateToPublicAccumulatedDataBuilder::empty();
    let mut revertible_builder = PrivateToPublicAccumulatedDataBuilder::empty();

    let note_hashes = data.note_hashes;
    for i in 0..note_hashes.max_len() {
        if i < note_hashes.len() {
            let note_hash = note_hashes.get_unchecked(i);
            if note_hash.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.note_hashes.push(note_hash.value());
            } else {
                revertible_builder.note_hashes.push(note_hash.value());
            }
        }
    }

    let nullifiers = data.nullifiers;
    for i in 0..nullifiers.max_len() {
        if i < nullifiers.len() {
            let nullifier = nullifiers.get_unchecked(i);
            if nullifier.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.nullifiers.push(nullifier.value());
            } else {
                revertible_builder.nullifiers.push(nullifier.value());
            }
        }
    }

    let l2_to_l1_msgs = data.l2_to_l1_msgs;
    for i in 0..l2_to_l1_msgs.max_len() {
        if i < l2_to_l1_msgs.len() {
            let msg = l2_to_l1_msgs.get_unchecked(i);
            if msg.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.l2_to_l1_msgs.push(msg.expose_to_public());
            } else {
                revertible_builder.l2_to_l1_msgs.push(msg.expose_to_public());
            }
        }
    }

    let private_logs = data.private_logs;
    for i in 0..private_logs.max_len() {
        if i < private_logs.len() {
            let private_log = private_logs.get_unchecked(i);
            if private_log.inner.counter < min_revertible_side_effect_counter {
                non_revertible_builder.private_logs.push(private_log.expose_to_public());
            } else {
                revertible_builder.private_logs.push(private_log.expose_to_public());
            }
        }
    }

    let contract_class_logs_hashes = data.contract_class_logs_hashes;
    for i in 0..contract_class_logs_hashes.max_len() {
        if i < contract_class_logs_hashes.len() {
            let contract_class_log_hash = contract_class_logs_hashes.get_unchecked(i);
            let public_log_hash = contract_class_log_hash.expose_to_public();
            if contract_class_log_hash.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.contract_class_logs_hashes.push(public_log_hash);
            } else {
                revertible_builder.contract_class_logs_hashes.push(public_log_hash);
            }
        }
    }

    let public_call_requests = data.public_call_requests;
    for i in 0..public_call_requests.max_len() {
        if i < public_call_requests.len() {
            let call_request = public_call_requests.get_unchecked(i);
            if call_request.counter < min_revertible_side_effect_counter {
                non_revertible_builder.public_call_requests.push(call_request.inner);
            } else {
                revertible_builder.public_call_requests.push(call_request.inner);
            }
        }
    }

    (non_revertible_builder.finish(), revertible_builder.finish())
}
