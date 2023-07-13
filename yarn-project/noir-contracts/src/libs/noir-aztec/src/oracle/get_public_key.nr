use crate::types::point::Point;
use crate::generators;
use dep::std::hash;

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
  
  // TODO: Remove zeroes when each point coordinate is converted to a single field
  let calculated_address = hash::pedersen_with_separator([pub_key_x, 0, pub_key_y, 0, partial_address], generators::ContractAddress)[0];
  assert(calculated_address == address);
  
  Point::new(pub_key_x, pub_key_y)
}