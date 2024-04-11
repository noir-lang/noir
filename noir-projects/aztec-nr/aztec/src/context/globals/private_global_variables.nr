use dep::protocol_types::traits::{Serialize, Empty};

global PRIVATE_GLOBAL_VARIABLES_LENGTH: u64 = 2;

// docs:start:private-global-variables
struct PrivateGlobalVariables {
    chain_id: Field,
    version: Field,
}
// docs:end:private-global-variables

// Note: public global vars are equal to the private ones 
impl Serialize<PRIVATE_GLOBAL_VARIABLES_LENGTH> for PrivateGlobalVariables {
    fn serialize(self) -> [Field; PRIVATE_GLOBAL_VARIABLES_LENGTH] {
        [self.chain_id, self.version]
    }
}

impl Empty for PrivateGlobalVariables {
    fn empty() -> Self {
        PrivateGlobalVariables {
            chain_id: 0,
            version: 0,
        }
    }
}