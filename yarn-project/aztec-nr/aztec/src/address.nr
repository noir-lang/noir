use dep::std::hash;
use crate::constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS;

fn compute_address(pub_key_x: Field, pub_key_y: Field, partial_address: Field) -> Field {
  hash::pedersen_with_separator([pub_key_x, pub_key_y, partial_address], GENERATOR_INDEX__CONTRACT_ADDRESS)[0]
}