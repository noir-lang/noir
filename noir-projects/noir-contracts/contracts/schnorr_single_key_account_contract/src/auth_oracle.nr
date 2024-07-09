use dep::authwit::auth_witness;
use dep::aztec::{
    protocol_types::{address::PartialAddress, utils::arr_copy_slice},
    keys::{PublicKeys, PUBLIC_KEYS_LENGTH}
};

struct AuthWitness {
    keys: PublicKeys,
    signature: [u8; 64],
    partial_address: PartialAddress,
}

impl AuthWitness {
    fn deserialize(values: [Field; 77]) -> Self {
        let mut signature = [0; 64];
        for i in 0..64 {
            signature[i] = values[i + PUBLIC_KEYS_LENGTH] as u8;
        }
        Self {
            keys: PublicKeys::deserialize(arr_copy_slice(values, [0; PUBLIC_KEYS_LENGTH], 0)),
            signature,
            partial_address: PartialAddress::from_field(values[76])
        }
    }
}

unconstrained pub fn get_auth_witness(message_hash: Field) -> AuthWitness {
    let witness: [Field; 77] = auth_witness::get_auth_witness(message_hash);
    AuthWitness::deserialize(witness)
}
