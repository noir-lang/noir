use dep::types::constants::GENERATOR_INDEX__GLOBAL_VARIABLES;
use dep::std::ops::Eq;
use dep::types::traits::Hash;

struct GlobalVariables {
    chain_id : Field,
    version : Field,
    block_number : Field,
    timestamp : Field,
}

impl Eq for GlobalVariables {
    fn eq(self, other : GlobalVariables) -> bool {
        (self.chain_id == other.chain_id) &
        (self.version == other.version) &
        (self.block_number == other.block_number) &
        (self.timestamp == other.timestamp)
    }
}

impl Hash for GlobalVariables {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.chain_id, 
            self.version, 
            self.block_number, 
            self.timestamp
            ],
            GENERATOR_INDEX__GLOBAL_VARIABLES,
            )
    }
}
