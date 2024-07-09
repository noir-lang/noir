use crate::keys::PublicKeys;
use dep::protocol_types::{address::{AztecAddress, PartialAddress}, point::Point};

#[oracle(getPublicKeysAndPartialAddress)]
unconstrained fn get_public_keys_and_partial_address_oracle(_address: AztecAddress) -> [Field; 13] {}

unconstrained fn get_public_keys_and_partial_address_oracle_wrapper(address: AztecAddress) -> [Field; 13] {
    get_public_keys_and_partial_address_oracle(address)
}

fn get_public_keys_and_partial_address(address: AztecAddress) -> (PublicKeys, PartialAddress) {
    let result = get_public_keys_and_partial_address_oracle_wrapper(address);

    let keys = PublicKeys {
        npk_m: Point::new(result[0], result[1], result[2] as bool),
        ivpk_m: Point::new(result[3], result[4], result[5] as bool),
        ovpk_m: Point::new(result[6], result[7], result[8] as bool),
        tpk_m: Point::new(result[9], result[10], result[11] as bool)
    };

    let partial_address = PartialAddress::from_field(result[12]);

    (keys, partial_address)
}
