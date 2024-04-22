use crate::{
    address::{
    eth_address::EthAddress, salted_initialization_hash::SaltedInitializationHash,
    aztec_address::AztecAddress
},
    constants::GENERATOR_INDEX__PARTIAL_ADDRESS, contract_class_id::ContractClassId,
    hash::pedersen_hash, traits::{ToField, FromField, Serialize, Deserialize}
};

global PARTIAL_ADDRESS_LENGTH = 1;

// Partial address
struct PartialAddress {
    inner : Field
}

impl ToField for PartialAddress {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl Serialize<PARTIAL_ADDRESS_LENGTH> for PartialAddress {
    fn serialize(self: Self) -> [Field; PARTIAL_ADDRESS_LENGTH] {
        [self.to_field()]
    }
}

impl Deserialize<PARTIAL_ADDRESS_LENGTH> for PartialAddress {
    fn deserialize(fields: [Field; PARTIAL_ADDRESS_LENGTH]) -> Self {
       PartialAddress { inner: fields[0] }
    }
}

impl PartialAddress {
    pub fn from_field(field: Field) -> Self {
        Self { inner: field }
    }

    pub fn compute(
        contract_class_id: ContractClassId,
        salt: Field,
        initialization_hash: Field,
        deployer: AztecAddress
    ) -> Self {
        PartialAddress::compute_from_salted_initialization_hash(
            contract_class_id,
            SaltedInitializationHash::compute(salt, initialization_hash, deployer)
        )
    }

    pub fn compute_from_salted_initialization_hash(
        contract_class_id: ContractClassId,
        salted_initialization_hash: SaltedInitializationHash
    ) -> Self {
        PartialAddress::from_field(
            pedersen_hash(
                [
            contract_class_id.to_field(),
            salted_initialization_hash.to_field()
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
