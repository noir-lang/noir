use dep::aztec::protocol_types::{
    contract_class::ContractClassId,
    address::{ AztecAddress, EthAddress, PublicKeysHash, PartialAddress },
    constants::{DEPLOYER_CONTRACT_INSTANCE_DEPLOYED_MAGIC_VALUE},
    traits::{Serialize}
};

// #[event]
struct ContractInstanceDeployed {
    address: AztecAddress,
    version: u8,
    salt: Field, 
    contract_class_id: ContractClassId, 
    initialization_hash: Field, 
    portal_contract_address: EthAddress, 
    public_keys_hash: PublicKeysHash, 
    universal_deploy: bool,
}

global CONTRACT_INSTANCE_DEPLOYED_SERIALIZED_SIZE: Field = 9;

impl Serialize<CONTRACT_INSTANCE_DEPLOYED_SERIALIZED_SIZE> for ContractInstanceDeployed {
    fn serialize(self: Self) -> [Field; CONTRACT_INSTANCE_DEPLOYED_SERIALIZED_SIZE] {
        [
            DEPLOYER_CONTRACT_INSTANCE_DEPLOYED_MAGIC_VALUE,
            self.address.to_field(),
            self.version as Field,
            self.salt,
            self.contract_class_id.to_field(),
            self.initialization_hash,
            self.portal_contract_address.to_field(),
            self.public_keys_hash.to_field(),
            self.universal_deploy as Field,
        ]
    }
}
