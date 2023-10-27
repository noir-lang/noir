use crate::address::EthAddress;
use crate::point::Point;
use dep::aztec::constants_gen;

struct ContractDeploymentData {
    deployer_public_key : Point,
    constructor_vk_hash : Field,
    function_tree_root : Field,
    contract_address_salt : Field,
    portal_contract_address : EthAddress,
}

impl ContractDeploymentData {
    
    fn assert_is_zero(self) {
        assert(self.constructor_vk_hash == 0);
        assert(self.function_tree_root == 0);
        assert(self.contract_address_salt == 0);
        assert(self.portal_contract_address.to_field() == 0);
        self.deployer_public_key.assert_is_zero();
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.deployer_public_key.x,
            self.deployer_public_key.y,
            self.constructor_vk_hash,
            self.function_tree_root,
            self.contract_address_salt,
            self.portal_contract_address.to_field()
        ], constants_gen::GENERATOR_INDEX__CONTRACT_DEPLOYMENT_DATA)
    }
}
