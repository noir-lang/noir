use crate::{
    crate::address::{eth_address::EthAddress, partial_address::PartialAddress, public_keys_hash::PublicKeysHash},
    constants::{AZTEC_ADDRESS_LENGTH, GENERATOR_INDEX__CONTRACT_ADDRESS_V1},
    contract_class_id::ContractClassId, hash::poseidon2_hash,
    traits::{Empty, FromField, ToField, Serialize, Deserialize}, utils
};

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

impl FromField for AztecAddress {
    fn from_field(value: Field) -> AztecAddress {
        AztecAddress { inner: value }
    }
}

impl Serialize<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn serialize(self: Self) -> [Field; AZTEC_ADDRESS_LENGTH] {
        [self.to_field()]
    }
}

impl Deserialize<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn deserialize(fields: [Field; AZTEC_ADDRESS_LENGTH]) -> Self {
        FromField::from_field(fields[0])
    }
}

impl AztecAddress {
    pub fn zero() -> Self {
        Self { inner: 0 }
    }

    pub fn compute(pub_keys_hash: PublicKeysHash, partial_address: PartialAddress) -> AztecAddress {
        AztecAddress::from_field(
            poseidon2_hash([pub_keys_hash.to_field(), partial_address.to_field(), GENERATOR_INDEX__CONTRACT_ADDRESS_V1])
        )
    }

    pub fn is_zero(self) -> bool {
        self.inner == 0
    }

    pub fn assert_is_zero(self) {
        assert(self.to_field() == 0);
    }

    pub fn conditional_assign(predicate: bool, lhs: Self, rhs: Self) -> Self {
        let result = utils::conditional_assign(predicate, rhs.to_field(), lhs.to_field());
        Self { inner: result }
    }
}

#[test]
fn compute_address_from_partial_and_pub_keys_hash() {
    let pub_keys_hash = PublicKeysHash::from_field(1);
    let partial_address = PartialAddress::from_field(2);

    let address = AztecAddress::compute(pub_keys_hash, partial_address);
    let expected_computed_address_from_partial_and_pubkey = 0x1b6ead051e7b42665064ca6cf1ec77da0a36d86e00d1ff6e44077966c0c3a9fa;
    assert(address.to_field() == expected_computed_address_from_partial_and_pubkey);
}

#[test]
fn from_field_to_field() {
    let address = AztecAddress { inner: 37 };
    assert_eq(FromField::from_field(address.to_field()), address);
}

#[test]
fn serde() {
    let address = AztecAddress { inner: 37 };
    assert_eq(Deserialize::deserialize(address.serialize()), address);
}
