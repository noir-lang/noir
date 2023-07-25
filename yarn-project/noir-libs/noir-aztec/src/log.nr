use crate::context::Context;
use crate::oracle;
use crate::types::point::Point;

fn emit_encrypted_log<N>(
    mut context: Context,
    contract_address: Field,
    storage_slot: Field,
    owner: Field,
    encryption_pub_key: Point,
    log: [Field; N],
) -> Context {
    let _ = oracle::logs::emit_encrypted_log(contract_address, storage_slot, owner, encryption_pub_key, log);
    context = context.accumulate_encrypted_logs(log);
    context
}

fn emit_unencrypted_log<T>(
    mut context: Context,
    log: T,
) -> Context {
    let _ = oracle::logs::emit_unencrypted_log(log);
    context = context.accumulate_unencrypted_logs(log);
    context
}
