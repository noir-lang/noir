use dep::types::abis::{
    accumulated_data::{
    private_accumulated_data_builder::PrivateAccumulatedDataBuilder,
    public_accumulated_data::PublicAccumulatedData,
    public_accumulated_data_builder::PublicAccumulatedDataBuilder
}
};

pub fn split_to_public(
    data: PrivateAccumulatedDataBuilder,
    min_revertible_side_effect_counter: u32
) -> (PublicAccumulatedData, PublicAccumulatedData) {
    assert(min_revertible_side_effect_counter != 0, "min_revertible_side_effect_counter must not be 0");

    let mut non_revertible_builder = PublicAccumulatedDataBuilder::empty();
    let mut revertible_builder = PublicAccumulatedDataBuilder::empty();

    let note_hashes = data.note_hashes;
    for i in 0..note_hashes.max_len() {
        if i < note_hashes.len() {
            let note_hash = note_hashes.get_unchecked(i);
            let public_note_hash = note_hash.expose_to_public();
            if note_hash.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.note_hashes.push(public_note_hash);
            } else {
                revertible_builder.note_hashes.push(public_note_hash);
            }
        }
    }

    let nullifiers = data.nullifiers;
    for i in 0..nullifiers.max_len() {
        if i < nullifiers.len() {
            let nullifier = nullifiers.get_unchecked(i);
            let public_nullifier = nullifier.expose_to_public();
            if nullifier.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.nullifiers.push(public_nullifier);
            } else {
                revertible_builder.nullifiers.push(public_nullifier);
            }
        }
    }

    let l2_to_l1_msgs = data.l2_to_l1_msgs;
    for i in 0..l2_to_l1_msgs.max_len() {
        if i < l2_to_l1_msgs.len() {
            let msg = l2_to_l1_msgs.get_unchecked(i);
            if msg.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.l2_to_l1_msgs.push(msg.message.content);
            } else {
                revertible_builder.l2_to_l1_msgs.push(msg.message.content);
            }
        }
    }

    let note_encrypted_logs_hashes = data.note_encrypted_logs_hashes;
    for i in 0..note_encrypted_logs_hashes.max_len() {
        if i < note_encrypted_logs_hashes.len() {
            let note_encrypted_log_hash = note_encrypted_logs_hashes.get_unchecked(i);
            let public_log_hash = note_encrypted_log_hash.expose_to_public();
            if note_encrypted_log_hash.counter < min_revertible_side_effect_counter {
                non_revertible_builder.note_encrypted_logs_hashes.push(public_log_hash);
            } else {
                revertible_builder.note_encrypted_logs_hashes.push(public_log_hash);
            }
        }
    }

    let encrypted_logs_hashes = data.encrypted_logs_hashes;
    for i in 0..encrypted_logs_hashes.max_len() {
        if i < encrypted_logs_hashes.len() {
            let encrypted_log_hash = encrypted_logs_hashes.get_unchecked(i);
            let public_log_hash = encrypted_log_hash.expose_to_public();
            if encrypted_log_hash.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.encrypted_logs_hashes.push(public_log_hash);
            } else {
                revertible_builder.encrypted_logs_hashes.push(public_log_hash);
            }
        }
    }

    let unencrypted_logs_hashes = data.unencrypted_logs_hashes;
    for i in 0..unencrypted_logs_hashes.max_len() {
        if i < unencrypted_logs_hashes.len() {
            let unencrypted_log_hash = unencrypted_logs_hashes.get_unchecked(i);
            let public_log_hash = unencrypted_log_hash.expose_to_public();
            if unencrypted_log_hash.counter() < min_revertible_side_effect_counter {
                non_revertible_builder.unencrypted_logs_hashes.push(public_log_hash);
            } else {
                revertible_builder.unencrypted_logs_hashes.push(public_log_hash);
            }
        }
    }

    let public_call_stack = data.public_call_stack;
    for i in 0..public_call_stack.max_len() {
        if i < public_call_stack.len() {
            let call_stack_item = public_call_stack.get_unchecked(i);
            // TODO: Hide the counter of a public call request.
            // let public_call_request = call_stack_item.expose_to_public();
            let public_call_request = call_stack_item;
            if call_stack_item.start_side_effect_counter < min_revertible_side_effect_counter {
                non_revertible_builder.public_call_stack.push(public_call_request);
            } else {
                revertible_builder.public_call_stack.push(public_call_request);
            }
        }
    }

    (non_revertible_builder.finish(), revertible_builder.finish())
}
