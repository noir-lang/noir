use dep::aztec::types::point::Point;

#[oracle(getAuthWitness)]
fn get_auth_witness_oracle(_message_hash: Field) -> [Field; 67] {}

struct AuthWitness {
    owner: Point,
    signature: [u8; 64],
    partial_address: Field,
}

impl AuthWitness {
    fn deserialize(values: [Field; 67]) -> Self {
        let mut signature = [0; 64];
        for i in 0..64 {
            signature[i] = values[i + 2] as u8;
        }
        Self {
            owner: Point::new(values[0], values[1]),
            signature,
            partial_address: values[66],
        }
    }
}

unconstrained fn get_auth_witness(message_hash: Field) -> AuthWitness {
    AuthWitness::deserialize(get_auth_witness_oracle(message_hash))
}