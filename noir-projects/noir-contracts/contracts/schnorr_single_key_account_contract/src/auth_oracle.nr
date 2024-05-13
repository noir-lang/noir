use dep::authwit::auth_witness;
use dep::aztec::{protocol_types::{address::PartialAddress, grumpkin_point::GrumpkinPoint}, keys::PublicKeys};

struct AuthWitness {
    keys: PublicKeys,
    signature: [u8; 64],
    partial_address: PartialAddress,
}

impl AuthWitness {
    fn deserialize(values: [Field; 73]) -> Self {
        let mut signature = [0; 64];
        for i in 0..64 {
            signature[i] = values[i + 8] as u8;
        }
        Self {
            keys: PublicKeys {
                npk_m: GrumpkinPoint::new(values[0], values[1]),
                ivpk_m: GrumpkinPoint::new(values[2], values[3]),
                ovpk_m: GrumpkinPoint::new(values[4], values[5]),
                tpk_m: GrumpkinPoint::new(values[6], values[7])
            },
            signature,
            partial_address: PartialAddress::from_field(values[72])
        }
    }
}

unconstrained pub fn get_auth_witness(message_hash: Field) -> AuthWitness {
    let witness: [Field; 73] = auth_witness::get_auth_witness(message_hash);
    AuthWitness::deserialize(witness)
}
