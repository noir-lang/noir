use crate::address::EthAddress;
use crate::constants::{
    CONTRACT_DEPLOYMENT_DATA_LENGTH,
    GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA,
};
use crate::hash::pedersen_hash;
use crate::point::Point;

// docs:start:contract-deployment-data
struct ContractDeploymentData {
    deployer_public_key : Point,
    constructor_vk_hash : Field,
    function_tree_root : Field,
    contract_address_salt : Field,
    portal_contract_address : EthAddress,
}
// docs:end:contract-deployment-data

impl ContractDeploymentData {
    fn serialize(self) -> [Field; CONTRACT_DEPLOYMENT_DATA_LENGTH] {
        [
            self.deployer_public_key.x,
            self.deployer_public_key.y,
            self.constructor_vk_hash,
            self.function_tree_root,
            self.contract_address_salt,
            self.portal_contract_address.to_field(),
        ]
    }

    fn assert_is_zero(self) {
        self.deployer_public_key.assert_is_zero();
        assert(self.constructor_vk_hash == 0);
        assert(self.function_tree_root == 0);
        assert(self.contract_address_salt == 0);
        self.portal_contract_address.assert_is_zero();
    }

    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA)
    }
}
