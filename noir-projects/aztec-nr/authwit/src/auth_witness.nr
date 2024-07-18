#[oracle(getAuthWitness)]
unconstrained fn get_auth_witness_oracle<let N: u32>(_message_hash: Field) -> [Field; N] {}

/**
 * Oracle wrapper to fetch an `auth_witness` for a given `message_hash` from the PXE.
 * 
 * @param message_hash The hash of the message for which the `auth_witness` is to be fetched.
 * @return The `auth_witness` for the given `message_hash` as Field array.
 */
unconstrained pub fn get_auth_witness<let N: u32>(message_hash: Field) -> [Field; N] {
    get_auth_witness_oracle(message_hash)
}
