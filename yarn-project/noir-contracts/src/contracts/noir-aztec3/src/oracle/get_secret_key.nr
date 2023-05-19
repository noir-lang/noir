use crate::types::point::Point;

#[oracle(getSecretKey)]
fn get_secret_key_oracle(_owner: Point) -> Field {}

unconstrained fn get_secret_key(owner: Point) -> Field {
    get_secret_key_oracle(owner)
}