use crate::context::{PrivateContext, PublicContext};
use crate::oracle;
use dep::protocol_types::{address::AztecAddress, grumpkin_point::GrumpkinPoint};

pub fn emit_encrypted_log<N>(
    context: &mut PrivateContext,
    contract_address: AztecAddress,
    storage_slot: Field,
    note_type_id: Field,
    encryption_pub_key: GrumpkinPoint,
    log: [Field; N]
) {
    let log_hash = oracle::logs::emit_encrypted_log(
        contract_address,
        storage_slot,
        note_type_id,
        encryption_pub_key,
        log
    );
    context.push_encrypted_log(log_hash);
}

pub fn emit_unencrypted_log<T>(context: &mut PublicContext, log: T) {
    let contract_address = context.this_address();
    let event_selector = 5; // TODO: compute actual event selector.
    let log_hash = oracle::logs::emit_unencrypted_log(contract_address, event_selector, log);
    context.push_unencrypted_log(log_hash);
}

// TODO: We might want to remove this since emitting unencrypted logs from private functions is violating privacy.
// --> might be a better approach to force devs to make a public function call that emits the log if needed then
// it would be less easy to accidentally leak information.
// If we decide to keep this function around would make sense to wait for traits and then merge it with emit_unencrypted_log.
pub fn emit_unencrypted_log_from_private<T>(context: &mut PrivateContext, log: T) {
    let contract_address = context.this_address();
    let event_selector = 5; // TODO: compute actual event selector.
    let log_hash = oracle::logs::emit_unencrypted_log(contract_address, event_selector, log);
    context.push_unencrypted_log(log_hash);
    // context.accumulate_unencrypted_logs(log);
}
