use crate::context::Context;
use crate::oracle;
use crate::types::point::Point;

fn emit_encrypted_log<N>(
    mut context: Context,
    contract_address: Field,
    storage_slot: Field,
    account: Point,
    log: [Field; N],
) -> Context {
    let hash = oracle::logs::emit_encrypted_log(contract_address, storage_slot, account, log);
    let mut len = log.len();
    if (hash[0] == 0) & (hash[1] == 0) {
        len = 0;
    };
    context = context.set_encrypted_logs_hash(hash);
    context = context.set_encrypted_log_preimages_length(len);
    context
}

fn emit_unencrypted_log<T>(
    mut context: Context,
    log: T,
) -> Context {
    let hash = oracle::logs::emit_unencrypted_log(log);
    // TODO
    let len = 0;
    // let len = log.len();
    context = context.set_unencrypted_logs_hash(hash);
    context = context.set_unencrypted_log_preimages_length(len);
    context
}
