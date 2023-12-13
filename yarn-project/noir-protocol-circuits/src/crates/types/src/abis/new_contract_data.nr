use crate::address::{AztecAddress, EthAddress};
use crate::constants::GENERATOR_INDEX__CONTRACT_LEAF;

struct NewContractData {
    contract_address: AztecAddress,
    portal_contract_address: EthAddress,
    function_tree_root: Field,
}

impl NewContractData {
    pub fn is_empty(self) -> bool {
        (self.contract_address.to_field() == 0) & 
        (self.portal_contract_address.to_field() == 0) & 
        (self.function_tree_root ==0)
    }

    pub fn eq(self, data: NewContractData) -> bool {
        data.contract_address.eq(self.contract_address)
            & data.portal_contract_address.eq(self.portal_contract_address)
            & (data.function_tree_root == self.function_tree_root)
    }

    pub fn default() -> Self {
        Self {
            contract_address : AztecAddress::default(),
            portal_contract_address : EthAddress::default(),
            function_tree_root : 0,
        }
    }

    pub fn hash(self) -> Field {
        if self.is_empty() {
            0 // We want to return 0 here since the contract_address is zero
        } else {
            dep::std::hash::pedersen_hash_with_separator([
                self.contract_address.to_field(),
                self.portal_contract_address.to_field(),
                self.function_tree_root,
            ], GENERATOR_INDEX__CONTRACT_LEAF)
        }
    }
}
