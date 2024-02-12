use dep::authwit::auth_witness;
use dep::aztec::protocol_types::{address::PartialAddress, grumpkin_point::GrumpkinPoint};

struct AuthWitness {
    owner: GrumpkinPoint,
    signature: [u8; 64],
    partial_address: PartialAddress,
}

impl AuthWitness {
    fn deserialize(values: [Field; 67]) -> Self {
        let mut signature = [0; 64];
        for i in 0..64 {
            signature[i] = values[i + 2] as u8;
        }
        Self {
            owner: GrumpkinPoint::new(values[0], values[1]),
            signature,
            partial_address: PartialAddress::from_field(values[66])
        }
    }
}

unconstrained pub fn get_auth_witness(message_hash: Field) -> AuthWitness {
    let witness: [Field; 67] = auth_witness::get_auth_witness(message_hash);
    AuthWitness::deserialize(witness)
}
