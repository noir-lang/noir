use crate::constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS;
use crate::hash::pedersen_hash;

pub fn compute_address(pub_key_x: Field, pub_key_y: Field, partial_address: Field) -> Field {
  pedersen_hash([pub_key_x, pub_key_y, partial_address], GENERATOR_INDEX__CONTRACT_ADDRESS)
}