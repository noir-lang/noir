use crate::{
    crate::address::{eth_address::EthAddress, partial_address::PartialAddress, public_keys_hash::PublicKeysHash},
    constants::{AZTEC_ADDRESS_LENGTH, GENERATOR_INDEX__CONTRACT_ADDRESS},
    contract_class_id::ContractClassId, hash::pedersen_hash, grumpkin_point::GrumpkinPoint,
    traits::{Empty, ToField, Serialize, Deserialize}, utils
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

impl Serialize<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn serialize(self: Self) -> [Field; AZTEC_ADDRESS_LENGTH] {
        [self.to_field()]
    }
}

impl Deserialize<AZTEC_ADDRESS_LENGTH> for AztecAddress {
    fn deserialize(fields: [Field; AZTEC_ADDRESS_LENGTH]) -> Self {
        AztecAddress::from_field(fields[0])
    }
}

impl AztecAddress {
    pub fn zero() -> Self {
        Self { inner: 0 }
    }

    pub fn from_field(field: Field) -> Self {
        Self { inner: field }
    }

    pub fn compute_from_public_key(
        pub_key: GrumpkinPoint,
        contract_class_id: ContractClassId,
        salt: Field,
        initialization_hash: Field,
        portal_contract_address: EthAddress,
        deployer: AztecAddress
    ) -> AztecAddress {
        AztecAddress::compute(
            PublicKeysHash::compute(pub_key),
            PartialAddress::compute(
                contract_class_id,
                salt,
                initialization_hash,
                portal_contract_address,
                deployer
            )
        )
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

    pub fn conditional_assign(predicate: bool, lhs: Self, rhs: Self) -> Self {
        let result = utils::conditional_assign(predicate, rhs.to_field(), lhs.to_field());
        Self { inner: result }
    }
}

#[test]
fn compute_address() {
    let point = GrumpkinPoint { x: 1, y: 2 };
    let contract_address_salt = 3;
    let contract_class_id = ContractClassId::from_field(4);
    let initialization_hash = 5;
    let portal_contract_address = EthAddress::from_field(6);
    let deployer = AztecAddress::from_field(7);

    let address = AztecAddress::compute_from_public_key(
        point,
        contract_class_id,
        contract_address_salt,
        initialization_hash,
        portal_contract_address,
        deployer
    );

    let expected_computed_address_from_preimage = 0x027ea2b41ced2ec9a98305984e96dd28518536a4628883ccdc06e38aa8997220;
    assert(address.to_field() == expected_computed_address_from_preimage);
}

#[test]
fn compute_address_from_partial_and_pubkey() {
    let point = GrumpkinPoint { x: 1, y: 2 };
    let partial_address = PartialAddress::from_field(3);

    let address = AztecAddress::compute(PublicKeysHash::compute(point), partial_address);
    let expected_computed_address_from_partial_and_pubkey = 0x0447f893197175723deb223696e2e96dbba1e707ee8507766373558877e74197;
    assert(address.to_field() == expected_computed_address_from_partial_and_pubkey);
}
