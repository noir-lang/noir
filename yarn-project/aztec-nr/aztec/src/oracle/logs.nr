use crate::types::point::Point;
use crate::constants_gen::NUM_FIELDS_PER_SHA256;

// TODO: Should take encrypted data.
#[oracle(emitEncryptedLog)]
fn emit_encrypted_log_oracle<N>(
    _contract_address: Field,
    _storage_slot: Field,
    _encryption_pub_key: Point,
    _preimage: [Field; N],
)  -> Field {}

unconstrained fn emit_encrypted_log<N>(
    contract_address: Field,
    storage_slot: Field,
    encryption_pub_key: Point,
    preimage: [Field; N],
) -> [Field; NUM_FIELDS_PER_SHA256] {
    [emit_encrypted_log_oracle(
        contract_address,
        storage_slot,
        encryption_pub_key,
        preimage,
    ), 0]
}

#[oracle(emitUnencryptedLog)]
fn emit_unencrypted_log_oracle<T>(_contract_address: Field, _event_selector: Field, _message: T)  -> Field {}

unconstrained fn emit_unencrypted_log<T>(contract_address: Field, event_selector: Field, message: T) -> [Field; NUM_FIELDS_PER_SHA256] {
    // https://github.com/AztecProtocol/aztec-packages/issues/885
    [emit_unencrypted_log_oracle(contract_address, event_selector, message), 0]
}