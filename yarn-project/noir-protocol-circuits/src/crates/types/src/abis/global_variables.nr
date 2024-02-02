use dep::std::cmp::Eq;
use crate::{
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
}

impl Serialize<GLOBAL_VARIABLES_LENGTH> for GlobalVariables {
    fn serialize(self) -> [Field; GLOBAL_VARIABLES_LENGTH] {
        [
            self.chain_id,
            self.version,
            self.block_number,
            self.timestamp
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
        }
    }
}

impl Eq for GlobalVariables {
    fn eq(self, other : GlobalVariables) -> bool {
        (self.chain_id == other.chain_id) &
        (self.version == other.version) &
        (self.block_number == other.block_number) &
        (self.timestamp == other.timestamp)
    }
}

impl Empty for GlobalVariables {
    fn empty() -> Self {
        Self {
            chain_id: 0,
            version: 0,
            block_number: 0,
            timestamp: 0,
        }
    }
}

#[test]
fn serialization_deserialization_smoke() {
    let vars: GlobalVariables = dep::std::unsafe::zeroed();
    let _serialized = vars.serialize();
    let _deserialized = GlobalVariables::deserialize(_serialized);
}
