use crate::{
    address::aztec_address::AztecAddress, constants::GENERATOR_INDEX__PARTIAL_ADDRESS,
    hash::poseidon2_hash_with_separator, traits::ToField,
};

// Salted initialization hash. Used in the computation of a partial address.
pub struct SaltedInitializationHash {
    pub inner: Field,
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

    pub fn compute(salt: Field, initialization_hash: Field, deployer: AztecAddress) -> Self {
        SaltedInitializationHash::from_field(poseidon2_hash_with_separator(
            [salt, initialization_hash, deployer.to_field()],
            GENERATOR_INDEX__PARTIAL_ADDRESS,
        ))
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}
