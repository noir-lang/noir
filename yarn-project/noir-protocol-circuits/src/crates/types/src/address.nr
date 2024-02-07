use crate::{
    constants::{
        GENERATOR_INDEX__CONTRACT_ADDRESS,
        GENERATOR_INDEX__PARTIAL_ADDRESS,
        GENERATOR_INDEX__CONSTRUCTOR
    },
    hash::pedersen_hash,
    contract_class::ContractClassId,
    utils,
    grumpkin_point::GrumpkinPoint,
};
use dep::std::cmp::Eq;
use crate::traits::{Empty, ToField, Serialize, Deserialize};
use crate::type_serialization::{ETH_ADDRESS_SERIALIZED_LEN, AZTEC_ADDRESS_SERIALIZED_LEN};

// Aztec address
struct AztecAddress {
    inner : Field
}

impl Eq for AztecAddress {
    fn eq(self, other : Self) -> bool {
        self.to_field() == other.to_field()
    }
}

impl Empty for AztecAddress {
    fn empty() -> Self {
        Self {
            inner : 0
        }
    }
}

impl ToField for AztecAddress {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl Serialize<AZTEC_ADDRESS_SERIALIZED_LEN> for AztecAddress {
    fn serialize(self: Self) -> [Field; AZTEC_ADDRESS_SERIALIZED_LEN] {
        [self.to_field()]
    }
}

impl Deserialize<AZTEC_ADDRESS_SERIALIZED_LEN> for AztecAddress {
    fn deserialize(fields: [Field; AZTEC_ADDRESS_SERIALIZED_LEN]) -> Self {
        AztecAddress::from_field(fields[0])
    }
}

impl AztecAddress {
    pub fn zero() -> Self {
        Self {
            inner: 0
        }
    }

    pub fn from_field(field : Field) -> Self {
        Self {
            inner : field
        }
    }

    pub fn compute_from_public_key(pub_key: GrumpkinPoint, contract_class_id: ContractClassId, salt: Field, initialization_hash: Field, portal_contract_address: EthAddress) -> AztecAddress {
        AztecAddress::compute(PublicKeysHash::compute(pub_key), PartialAddress::compute(contract_class_id, salt, initialization_hash, portal_contract_address))
    }

    pub fn compute(pub_keys_hash: PublicKeysHash, partial_address: PartialAddress) -> AztecAddress {
        AztecAddress::from_field(
            pedersen_hash(
                [pub_keys_hash.to_field(), partial_address.to_field()],
                GENERATOR_INDEX__CONTRACT_ADDRESS
            )
        )
    }

    pub fn is_zero(self) -> bool {
        self.inner == 0
    }
    
    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }

    pub fn conditional_assign(predicate: bool, lhs : Self, rhs : Self) -> Self{
        let result = utils::conditional_assign(predicate, rhs.to_field(), lhs.to_field());
        Self {
            inner : result
        }
    }
}

struct EthAddress{
    inner : Field
}

impl Eq for EthAddress {
    fn eq(self, other : Self) -> bool {
        self.to_field() == other.to_field()
    }
}

impl Empty for EthAddress {
    fn empty() -> Self {
        Self {
            inner : 0
        }
    }
}

impl ToField for EthAddress {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl Serialize<ETH_ADDRESS_SERIALIZED_LEN> for EthAddress {
    fn serialize(self: Self) -> [Field; ETH_ADDRESS_SERIALIZED_LEN] {
        [self.inner]
    }
}

impl Deserialize<ETH_ADDRESS_SERIALIZED_LEN> for EthAddress {
    fn deserialize(fields: [Field; ETH_ADDRESS_SERIALIZED_LEN]) -> Self {
        Self {
            inner: fields[0]
        }
    }
}

impl EthAddress{
    pub fn zero() -> Self {
        Self {
            inner: 0
        }
    }

    pub fn from_field(field : Field) -> Self {
        Self {
            inner : field
        }
    }

    pub fn is_zero(self) -> bool {
        self.inner == 0
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }

    pub fn conditional_assign(predicate: bool, lhs : Self, rhs : Self) -> Self{
        let result = utils::conditional_assign(predicate, rhs.to_field(), lhs.to_field());
        Self {
            inner : result
        }
    }
}

// Partial address
struct PartialAddress {
    inner : Field
}

impl ToField for PartialAddress {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl PartialAddress {
    pub fn from_field(field : Field) -> Self {
        Self {
            inner : field
        }
    }

    pub fn compute(contract_class_id: ContractClassId, salt: Field, initialization_hash: Field, portal_contract_address: EthAddress) -> Self {
        PartialAddress::compute_from_salted_initialization_hash(contract_class_id, SaltedInitializationHash::compute(salt, initialization_hash, portal_contract_address))
    }

    pub fn compute_from_salted_initialization_hash(contract_class_id: ContractClassId, salted_initialization_hash: SaltedInitializationHash) -> Self {
        PartialAddress::from_field(
            pedersen_hash([
                contract_class_id.to_field(),
                salted_initialization_hash.to_field()
            ], GENERATOR_INDEX__PARTIAL_ADDRESS)
        )
    }


    pub fn to_field(self) -> Field {
        self.inner
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}

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
    pub fn from_field(field : Field) -> Self {
        Self {
            inner : field
        }
    }

    pub fn compute(salt : Field, initialization_hash: Field, portal_contract_address: EthAddress) -> Self {
        SaltedInitializationHash::from_field(
            pedersen_hash([
                salt,
                initialization_hash,
                portal_contract_address.to_field(),
            ], GENERATOR_INDEX__PARTIAL_ADDRESS)
        )
    }

    pub fn to_field(self) -> Field {
        self.inner
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}

// Public keys hash. Used in the computation of an address.
struct PublicKeysHash {
    inner: Field
}

impl ToField for PublicKeysHash {
    fn to_field(self) -> Field {
        self.inner
    }
}

impl Serialize<1> for PublicKeysHash {
    fn serialize(self: Self) -> [Field; 1] {
        [self.to_field()]
    }
}

impl Deserialize<1> for PublicKeysHash {
    fn deserialize(fields: [Field; 1]) -> Self {
        PublicKeysHash::from_field(fields[0])
    }
}

impl PublicKeysHash {
    pub fn from_field(field : Field) -> Self {
        Self {
            inner : field
        }
    }

    pub fn compute(public_key: GrumpkinPoint) -> Self {
        PublicKeysHash::from_field(
            pedersen_hash([
                public_key.x,
                public_key.y,
            ], GENERATOR_INDEX__PARTIAL_ADDRESS)
        )
    }

    pub fn to_field(self) -> Field {
        self.inner
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }
}

pub fn compute_initialization_hash(selector: Field, args_hash: Field) -> Field {
    pedersen_hash(
        [
        selector,
        args_hash
    ],
        GENERATOR_INDEX__CONSTRUCTOR
    )
}
