use crate::address::EthAddress;
use crate::constants::{
    CONTRACT_DEPLOYMENT_DATA_LENGTH,
    GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA,
};
use crate::hash::pedersen_hash;
use crate::grumpkin_point::GrumpkinPoint;
use crate::traits::{Hash, Serialize};

// docs:start:contract-deployment-data
struct ContractDeploymentData {
    public_key : GrumpkinPoint,
    initialization_hash : Field,
    contract_class_id : Field,
    contract_address_salt : Field,
    portal_contract_address : EthAddress,
}
// docs:end:contract-deployment-data

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
            self.contract_class_id,
            self.contract_address_salt,
            self.portal_contract_address.to_field(),
        ]
    }
}

impl ContractDeploymentData {
    
    fn assert_is_zero(self) {
        self.public_key.assert_is_zero();
        assert(self.initialization_hash == 0);
        assert(self.contract_class_id == 0);
        assert(self.contract_address_salt == 0);
        self.portal_contract_address.assert_is_zero();
    }

}
