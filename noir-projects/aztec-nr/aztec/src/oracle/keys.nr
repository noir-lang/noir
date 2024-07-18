use crate::keys::PublicKeys;
use dep::protocol_types::{address::{AztecAddress, PartialAddress}, point::Point};

#[oracle(getPublicKeysAndPartialAddress)]
unconstrained fn get_public_keys_and_partial_address_oracle(_address: AztecAddress) -> [Field; 13] {}

unconstrained fn get_public_keys_and_partial_address_oracle_wrapper(address: AztecAddress) -> [Field; 13] {
    get_public_keys_and_partial_address_oracle(address)
}

pub fn get_public_keys_and_partial_address(address: AztecAddress) -> (PublicKeys, PartialAddress) {
    let result = get_public_keys_and_partial_address_oracle_wrapper(address);

    let keys = PublicKeys {
        npk_m: Point { x: result[0], y: result[1], is_infinite: result[2] as bool },
        ivpk_m: Point { x: result[3], y: result[4], is_infinite: result[5] as bool },
        ovpk_m: Point { x: result[6], y: result[7], is_infinite: result[8] as bool },
        tpk_m: Point { x: result[9], y: result[10], is_infinite: result[11] as bool }
    };

    let partial_address = PartialAddress::from_field(result[12]);

    (keys, partial_address)
}
