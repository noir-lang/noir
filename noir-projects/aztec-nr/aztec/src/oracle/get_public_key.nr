use dep::protocol_types::{address::AztecAddress, grumpkin_point::GrumpkinPoint};
use crate::oracle::keys::get_public_keys_and_partial_address;

// To be nuked in my next PR: https://github.com/AztecProtocol/aztec-packages/pull/6219
pub fn get_public_key(address: AztecAddress) -> GrumpkinPoint {
    let result = get_public_keys_and_partial_address(address);
    result.0[1]
}
