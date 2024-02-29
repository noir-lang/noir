use crate::{
    address::EthAddress, contract_class_id::ContractClassId,
    constants::{CONTRACT_DEPLOYMENT_DATA_LENGTH, GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA},
    grumpkin_point::GrumpkinPoint, hash::pedersen_hash, traits::{Deserialize, Hash, Serialize}
};

// docs:start:contract-deployment-data
struct ContractDeploymentData {
    public_key : GrumpkinPoint,
    initialization_hash : Field,
    contract_class_id : ContractClassId,
    contract_address_salt : Field,
    portal_contract_address : EthAddress,
}
// docs:end:contract-deployment-data

impl Eq for ContractDeploymentData {
    fn eq(self, other: Self) -> bool {
        self.public_key.eq(other.public_key) &
        self.initialization_hash.eq(other.initialization_hash) &
        self.contract_class_id.eq(other.contract_class_id) &
        self.contract_address_salt.eq(other.contract_address_salt) &
        self.portal_contract_address.eq(other.portal_contract_address)
    }
}

impl Hash for ContractDeploymentData {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA)
    }
}

impl Serialize<CONTRACT_DEPLOYMENT_DATA_LENGTH> for ContractDeploymentData {
    fn serialize(self) -> [Field; CONTRACT_DEPLOYMENT_DATA_LENGTH] {
        [
            self.public_key.x,
            self.public_key.y,
            self.initialization_hash,
            self.contract_class_id.to_field(),
            self.contract_address_salt,
            self.portal_contract_address.to_field(),
        ]
    }
}

impl Deserialize<CONTRACT_DEPLOYMENT_DATA_LENGTH> for ContractDeploymentData {
    fn deserialize(serialized: [Field; CONTRACT_DEPLOYMENT_DATA_LENGTH]) -> Self {
        Self {
            public_key: GrumpkinPoint {
                x: serialized[0],
                y: serialized[1],
            },
            initialization_hash: serialized[2],
            contract_class_id: ContractClassId::from_field(serialized[3]),
            contract_address_salt: serialized[4],
            portal_contract_address: EthAddress::from_field(serialized[5]),
        }
    }
}

impl ContractDeploymentData {
    fn assert_is_zero(self) {
        self.public_key.assert_is_zero();
        assert(self.initialization_hash == 0);
        self.contract_class_id.assert_is_zero();
        assert(self.contract_address_salt == 0);
        self.portal_contract_address.assert_is_zero();
    }
}

#[test]
fn serialization_of_empty() {
    let data: ContractDeploymentData = dep::std::unsafe::zeroed();
    let serialized = data.serialize();
    let deserialized = ContractDeploymentData::deserialize(serialized);
    assert(data.eq(deserialized));
}

#[test]
fn empty_hash() {
    let data: ContractDeploymentData = dep::std::unsafe::zeroed();
    let hash = data.hash();

    // Value from contract_deployment_data.test.ts "computes empty item hash" test
    assert_eq(hash, 0x0e7babf59de8dfc7f5992cd34fb0066105d07ce67f68fc33d4a0a6a933a30405);
}
