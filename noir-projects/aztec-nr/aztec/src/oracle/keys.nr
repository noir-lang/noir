use crate::keys::PublicKeys;
use dep::protocol_types::{address::{AztecAddress, PartialAddress}, grumpkin_point::GrumpkinPoint};

#[oracle(getPublicKeysAndPartialAddress)]
fn get_public_keys_and_partial_address_oracle(_address: AztecAddress) -> [Field; 9] {}

unconstrained fn get_public_keys_and_partial_address_oracle_wrapper(address: AztecAddress) -> [Field; 9] {
    get_public_keys_and_partial_address_oracle(address)
}

fn get_public_keys_and_partial_address(address: AztecAddress) -> (PublicKeys, PartialAddress) {
    let result = get_public_keys_and_partial_address_oracle_wrapper(address);

    let keys = PublicKeys {
        npk_m: GrumpkinPoint::new(result[0], result[1]),
        ivpk_m: GrumpkinPoint::new(result[2], result[3]),
        ovpk_m: GrumpkinPoint::new(result[4], result[5]),
        tpk_m: GrumpkinPoint::new(result[6], result[7])
    };

    let partial_address = PartialAddress::from_field(result[8]);

    (keys, partial_address)
}
