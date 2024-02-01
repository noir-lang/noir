use crate::address::{AztecAddress, EthAddress};
use crate::contract_class::ContractClassId;
use crate::constants::GENERATOR_INDEX__CONTRACT_LEAF;
use dep::std::cmp::Eq;
use crate::traits::{Empty, Hash};

struct NewContractData {
    contract_address: AztecAddress,
    portal_contract_address: EthAddress,
    contract_class_id: ContractClassId,
}

impl Eq for NewContractData {
    fn eq(self, data: NewContractData) -> bool {
        data.contract_address.eq(self.contract_address)
            & data.portal_contract_address.eq(self.portal_contract_address)
            & data.contract_class_id.eq(self.contract_class_id)
    }
}

impl Empty for NewContractData {
    fn empty() -> Self {
        Self {
            contract_address : AztecAddress::empty(),
            portal_contract_address : EthAddress::empty(),
            contract_class_id: ContractClassId::from_field(0),
        }
    }
}

impl Hash for NewContractData {
    fn hash(self) -> Field {
        if self.is_empty() {
            0 // We want to return 0 here since the contract_address is zero
        } else {
            dep::std::hash::pedersen_hash_with_separator([
                self.contract_address.to_field(),
                self.portal_contract_address.to_field(),
                self.contract_class_id.to_field(),
            ], GENERATOR_INDEX__CONTRACT_LEAF)
        }
    }
}

impl NewContractData {
    pub fn is_empty(self) -> bool {
        (self.contract_address.to_field() == 0) & 
        (self.portal_contract_address.to_field() == 0) & 
        (self.contract_class_id.to_field() == 0)
    }

}
