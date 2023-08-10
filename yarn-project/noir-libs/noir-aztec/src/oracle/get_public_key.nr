use crate::types::point::Point;
use dep::std::hash;
use crate::constants_gen::GENERATOR_INDEX__CONTRACT_ADDRESS;

#[oracle(getPublicKey)]
fn get_public_key_oracle(_address: Field) -> [Field; 3] {}

unconstrained fn get_public_key_internal(address: Field) -> [Field; 3] {
  get_public_key_oracle(address)
}

fn get_public_key(address: Field) -> Point {
  let result = get_public_key_internal(address);
  let pub_key_x = result[0];
  let pub_key_y = result[1];
  let partial_address = result[2];
  
  let calculated_address = hash::pedersen_with_separator([pub_key_x, pub_key_y, partial_address], GENERATOR_INDEX__CONTRACT_ADDRESS)[0];
  assert(calculated_address == address);
  
  Point::new(pub_key_x, pub_key_y)
}
