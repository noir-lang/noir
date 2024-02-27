use crate::{
    address::{
    aztec_address::AztecAddress, eth_address::EthAddress, partial_address::PartialAddress,
    public_keys_hash::PublicKeysHash
},
    contract_class_id::ContractClassId,
    constants::{GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA, CONTRACT_INSTANCE_LENGTH},
    traits::{Deserialize, Hash, Serialize}
};

struct ContractInstance {
    salt : Field,
    deployer: AztecAddress,
    contract_class_id : ContractClassId,
    initialization_hash : Field,
    portal_contract_address : EthAddress,
    public_keys_hash : PublicKeysHash,
}

impl Eq for ContractInstance {
    fn eq(self, other: Self) -> bool {
        self.public_keys_hash.eq(other.public_keys_hash) &
        self.initialization_hash.eq(other.initialization_hash) &
        self.contract_class_id.eq(other.contract_class_id) &
        self.salt.eq(other.salt) &
        self.portal_contract_address.eq(other.portal_contract_address)
    }
}

impl Serialize<CONTRACT_INSTANCE_LENGTH> for ContractInstance {
    fn serialize(self) -> [Field; CONTRACT_INSTANCE_LENGTH] {
        [
            self.salt,
            self.deployer.to_field(),
            self.contract_class_id.to_field(),
            self.initialization_hash,
            self.portal_contract_address.to_field(),
            self.public_keys_hash.to_field()
        ]
    }
}

impl Deserialize<CONTRACT_INSTANCE_LENGTH> for ContractInstance {
    fn deserialize(serialized: [Field; CONTRACT_INSTANCE_LENGTH]) -> Self {
        Self {
            salt: serialized[0],
            deployer: AztecAddress::from_field(serialized[1]),
            contract_class_id: ContractClassId::from_field(serialized[2]),
            initialization_hash: serialized[3],
            portal_contract_address: EthAddress::from_field(serialized[4]),
            public_keys_hash: PublicKeysHash::from_field(serialized[5]),
        }
    }
}

impl Hash for ContractInstance {
    fn hash(self) -> Field {
        self.to_address().to_field()
    }
}

impl ContractInstance {
    fn to_address(self) -> AztecAddress {
        AztecAddress::compute(
            self.public_keys_hash,
            PartialAddress::compute(
                self.contract_class_id,
                self.salt,
                self.initialization_hash,
                self.portal_contract_address
            )
        )
    }
}
