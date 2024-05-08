use dep::protocol_types::{address::{AztecAddress, PartialAddress}, grumpkin_point::GrumpkinPoint};

use crate::hash::poseidon2_hash;

#[oracle(getPublicKeysAndPartialAddress)]
fn get_public_keys_and_partial_address_oracle(_address: AztecAddress) -> [Field; 9] {}

unconstrained fn get_public_keys_and_partial_address_oracle_wrapper(address: AztecAddress) -> [Field; 9] {
    get_public_keys_and_partial_address_oracle(address)
}

fn get_public_keys_and_partial_address(address: AztecAddress) -> ([GrumpkinPoint; 4], PartialAddress) {
    let result = get_public_keys_and_partial_address_oracle_wrapper(address);

    let nullifier_pub_key = GrumpkinPoint::new(result[0], result[1]);
    let incoming_pub_key = GrumpkinPoint::new(result[2], result[3]);
    let outgoing_pub_key = GrumpkinPoint::new(result[4], result[5]);
    let tagging_pub_key = GrumpkinPoint::new(result[6], result[7]);
    let partial_address = PartialAddress::from_field(result[8]);

    ([nullifier_pub_key, incoming_pub_key, outgoing_pub_key, tagging_pub_key], partial_address)
}
