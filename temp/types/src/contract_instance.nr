use crate::{
    address::{aztec_address::AztecAddress, partial_address::PartialAddress},
    constants::CONTRACT_INSTANCE_LENGTH,
    contract_class_id::ContractClassId,
    public_keys::PublicKeys,
    traits::{Deserialize, FromField, Hash, Serialize, ToField},
};

pub struct ContractInstance {
    pub salt: Field,
    pub deployer: AztecAddress,
    pub contract_class_id: ContractClassId,
    pub initialization_hash: Field,
    pub public_keys: PublicKeys,
}

impl Eq for ContractInstance {
    fn eq(self, other: Self) -> bool {
        self.public_keys.eq(other.public_keys)
            & self.initialization_hash.eq(other.initialization_hash)
            & self.contract_class_id.eq(other.contract_class_id)
            & self.salt.eq(other.salt)
    }
}

impl Serialize<CONTRACT_INSTANCE_LENGTH> for ContractInstance {
    fn serialize(self) -> [Field; CONTRACT_INSTANCE_LENGTH] {
        let public_keys_serialized = self.public_keys.serialize();
        [
            self.salt,
            self.deployer.to_field(),
            self.contract_class_id.to_field(),
            self.initialization_hash,
            public_keys_serialized[0],
            public_keys_serialized[1],
            public_keys_serialized[2],
            public_keys_serialized[3],
            public_keys_serialized[4],
            public_keys_serialized[5],
            public_keys_serialized[6],
            public_keys_serialized[7],
            public_keys_serialized[8],
            public_keys_serialized[9],
            public_keys_serialized[10],
            public_keys_serialized[11],
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
            public_keys: PublicKeys::deserialize([
                serialized[4],
                serialized[5],
                serialized[6],
                serialized[7],
                serialized[8],
                serialized[9],
                serialized[10],
                serialized[11],
                serialized[12],
                serialized[13],
                serialized[14],
                serialized[15],
            ]),
        }
    }
}

impl Hash for ContractInstance {
    fn hash(self) -> Field {
        self.to_address().to_field()
    }
}

impl ContractInstance {
    pub fn to_address(self) -> AztecAddress {
        AztecAddress::compute(
            self.public_keys,
            PartialAddress::compute(
                self.contract_class_id,
                self.salt,
                self.initialization_hash,
                self.deployer,
            ),
        )
    }
}
