use crate::{
    address::eth_address::EthAddress, constants::GENERATOR_INDEX__PARTIAL_ADDRESS, hash::pedersen_hash,
    traits::ToField
};

// Salted initialization hash. Used in the computation of a partial address.
struct SaltedInitializationHash {
    inner: Field
}

impl ToField for SaltedInitializationHash {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl SaltedInitializationHash {
    pub fn from_field(field: Field) -> Self {
        Self { inner: field }
    }

    pub fn compute(salt: Field, initialization_hash: Field, portal_contract_address: EthAddress) -> Self {
        SaltedInitializationHash::from_field(
            pedersen_hash(
                [
            salt,
            initialization_hash,
            portal_contract_address.to_field()
        ],
                GENERATOR_INDEX__PARTIAL_ADDRESS
            )
        )
    }

    pub fn to_field(self) -> Field {
        self.inner
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}
