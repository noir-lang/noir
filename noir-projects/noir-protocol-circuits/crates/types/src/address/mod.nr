mod aztec_address;
mod eth_address;
mod partial_address;
mod public_keys_hash;
mod salted_initialization_hash;

use crate::address::aztec_address::AztecAddress;
use crate::address::eth_address::EthAddress;
use crate::address::partial_address::PartialAddress;
use crate::address::public_keys_hash::PublicKeysHash;
use crate::address::salted_initialization_hash::SaltedInitializationHash;

use crate::{constants::GENERATOR_INDEX__CONSTRUCTOR, hash::pedersen_hash};

pub fn compute_initialization_hash(selector: Field, args_hash: Field) -> Field {
    pedersen_hash(
        [
        selector,
        args_hash
    ],
        GENERATOR_INDEX__CONSTRUCTOR
    )
}
