#[oracle(getAuthWitness)]
fn get_auth_witness_oracle<N>(_message_hash: Field) -> [Field; N] {}

unconstrained fn get_auth_witness<N>(message_hash: Field) -> [Field; N] {
    get_auth_witness_oracle(message_hash)
}