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
