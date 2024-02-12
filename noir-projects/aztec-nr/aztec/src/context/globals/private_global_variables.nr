use dep::protocol_types::traits::Serialize;

global PRIVATE_GLOBAL_VARIABLES_LENGTH: Field = 2;

// docs:start:private-global-variables
struct PrivateGlobalVariables {
    chain_id: Field,
    version: Field,
}
// docs:end:private-global-variables

impl Serialize<PRIVATE_GLOBAL_VARIABLES_LENGTH> for PrivateGlobalVariables {
    fn serialize(self) -> [Field; PRIVATE_GLOBAL_VARIABLES_LENGTH] {
        [self.chain_id, self.version]
    }
}
