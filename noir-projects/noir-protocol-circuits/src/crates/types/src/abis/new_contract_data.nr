use crate::address::{AztecAddress, EthAddress};
use crate::contract_class::ContractClassId;
use crate::constants::{GENERATOR_INDEX__CONTRACT_LEAF, NEW_CONTRACT_DATA_LENGTH};
use dep::std::cmp::Eq;
use crate::traits::{Empty, Serialize, Hash, Deserialize};
use crate::hash::pedersen_hash;

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

impl Serialize<NEW_CONTRACT_DATA_LENGTH> for NewContractData {
    fn serialize(self) -> [Field; NEW_CONTRACT_DATA_LENGTH] {
        [
            self.contract_address.to_field(),
            self.portal_contract_address.to_field(),
            self.contract_class_id.to_field(),
        ]
    }
}

impl Deserialize<NEW_CONTRACT_DATA_LENGTH> for NewContractData {
    fn deserialize(serialized: [Field; NEW_CONTRACT_DATA_LENGTH]) -> Self {
        Self {
            contract_address: AztecAddress::from_field(serialized[0]),
            portal_contract_address: EthAddress::from_field(serialized[1]),
            contract_class_id: ContractClassId::from_field(serialized[2]),
        }
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
            pedersen_hash(self.serialize(), GENERATOR_INDEX__CONTRACT_LEAF)
        }
    }
}

impl NewContractData {
    pub fn is_empty(self) -> bool {
        (self.contract_address.to_field() == 0)
            & (self.portal_contract_address.to_field() == 0)
            & (self.contract_class_id.to_field() == 0)
    }
}
