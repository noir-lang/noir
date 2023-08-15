use crate::context::PrivateContext;
use crate::oracle;
use crate::types::point::Point;

fn emit_encrypted_log<N>(
    context: &mut PrivateContext,
    contract_address: Field,
    storage_slot: Field,
    encryption_pub_key: Point,
    log: [Field; N],
) {
    let _ = oracle::logs::emit_encrypted_log(contract_address, storage_slot, encryption_pub_key, log);
    context.accumulate_encrypted_logs(log);
}

fn emit_unencrypted_log<T>(
    context: &mut PrivateContext,
    log: T,
) {
    let _ = oracle::logs::emit_unencrypted_log(log);
    context.accumulate_unencrypted_logs(log);
}
