use dep::aztec::protocol_types::abis::log_hash::LogHash;
use dep::aztec::oracle::logs::emit_unencrypted_log_private_internal;
use dep::aztec::hash::compute_unencrypted_log_hash;
use dep::aztec::context::PrivateContext;

fn emit_nonce_as_unencrypted_log(context: &mut PrivateContext, nonce: Field) {
    let counter = context.next_counter();
    let event_type_id = 0;
    let log_slice = nonce.to_be_bytes_arr();
    let log_hash = compute_unencrypted_log_hash(context.this_address(), event_type_id, nonce);
    // 44 = addr (32) + selector (4) + raw log len (4) + processed log len (4)
    let len = 44 + log_slice.len().to_field();
    let side_effect = LogHash { value: log_hash, counter, length: len };
    context.unencrypted_logs_hashes.push(side_effect);
    let _void = emit_unencrypted_log_private_internal(context.this_address(), event_type_id, nonce, counter);
}
