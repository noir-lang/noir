use crate::types::point::Point;
use crate::oracle::get_public_key::get_public_key;

#[oracle(getSecretKey)]
fn get_secret_key_oracle(_owner: Point) -> Field {}

unconstrained fn get_secret_key(owner: Field) -> Field {
    let owner_nullifying_public_key = get_public_key(owner);
    get_secret_key_oracle(owner_nullifying_public_key)
}