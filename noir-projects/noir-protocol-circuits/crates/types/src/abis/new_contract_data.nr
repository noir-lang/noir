use crate::address::{AztecAddress, EthAddress};
use crate::contract_class_id::ContractClassId;
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

mod tests {
    use crate::{
        abis::new_contract_data::NewContractData, address::{AztecAddress, EthAddress},
        contract_class_id::ContractClassId
    };

    // Matches makeNewContractData in factories.ts
    fn make_new_contract_data(seed: Field) -> NewContractData {
        NewContractData {
            contract_address: AztecAddress::from_field(seed),
            portal_contract_address: EthAddress::from_field(seed + 1),
            contract_class_id: ContractClassId::from_field(seed + 2)
        }
    }

    #[test]
    fn serialization_of_empty() {
        let data: NewContractData = dep::std::unsafe::zeroed();
        let serialized = data.serialize();
        let deserialized = NewContractData::deserialize(serialized);
        assert(data.eq(deserialized));
    }

    #[test]
    fn empty_hash_is_zero() {
        let data: NewContractData = dep::std::unsafe::zeroed();
        let hash = data.hash();

        // Value from new_contract_data.test.ts "computes empty hash" test
        assert_eq(hash, 0x0000000000000000000000000000000000000000000000000000000000000000);
    }

    #[test]
    fn hash_matches() {
        let data = make_new_contract_data(5);
        let hash = data.hash();

        // Value from new_contract_data.test.ts "hash matches" test
        assert_eq(hash, 0x2359862482fb58cf5a5ddab56eb6d49908bc811353f3fd6fe9fd057c525cbf0b);
    }
}
