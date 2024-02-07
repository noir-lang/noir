use dep::std::cmp::Eq;
use crate::{
    address::{
        AztecAddress,
        EthAddress,
    },
    constants::{
        GENERATOR_INDEX__GLOBAL_VARIABLES,
        GLOBAL_VARIABLES_LENGTH,
    },
    traits::{
        Deserialize,
        Empty,
        Hash,
        Serialize,
    },
};

struct GlobalVariables {
    chain_id : Field,
    version : Field,
    block_number : Field,
    timestamp : Field,
    coinbase : EthAddress,
    fee_recipient : AztecAddress,
}

impl Serialize<GLOBAL_VARIABLES_LENGTH> for GlobalVariables {
    fn serialize(self) -> [Field; GLOBAL_VARIABLES_LENGTH] {
        [
            self.chain_id,
            self.version,
            self.block_number,
            self.timestamp,
            self.coinbase.to_field(),
            self.fee_recipient.to_field(),
        ]
    }
}

impl Deserialize<GLOBAL_VARIABLES_LENGTH> for GlobalVariables {
    fn deserialize(serialized: [Field; GLOBAL_VARIABLES_LENGTH]) -> GlobalVariables {
        GlobalVariables {
            chain_id: serialized[0],
            version: serialized[1],
            block_number: serialized[2],
            timestamp: serialized[3],
            coinbase: EthAddress::from_field(serialized[4]),
            fee_recipient: AztecAddress::from_field(serialized[5]),
        }
    }
}

impl Eq for GlobalVariables {
    fn eq(self, other : GlobalVariables) -> bool {
        (self.chain_id == other.chain_id) &
        (self.version == other.version) &
        (self.block_number == other.block_number) &
        (self.timestamp == other.timestamp) &
        (self.coinbase == other.coinbase) &
        (self.fee_recipient == other.fee_recipient)
    }
}

impl Empty for GlobalVariables {
    fn empty() -> Self {
        Self {
            chain_id: 0,
            version: 0,
            block_number: 0,
            timestamp: 0,
            coinbase: EthAddress::empty(),
            fee_recipient: AztecAddress::empty(),
        }
    }
}

#[test]
fn serialization_of_empty() {
    let vars: GlobalVariables = dep::std::unsafe::zeroed();
    let _serialized = vars.serialize();
    let _deserialized = GlobalVariables::deserialize(_serialized);
}
