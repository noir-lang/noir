use dep::aztec::constants_gen;

struct GlobalVariables {
    chain_id : Field,
    version : Field,
    block_number : Field,
    timestamp : Field,
}

impl GlobalVariables {

    pub fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.chain_id, 
            self.version, 
            self.block_number, 
            self.timestamp
            ],
            constants_gen::GENERATOR_INDEX__GLOBAL_VARIABLES,
            )
    }

    pub fn eq(self, other : GlobalVariables) -> bool {
        (self.chain_id == other.chain_id) &
        (self.version == other.version) &
        (self.block_number == other.block_number) &
        (self.timestamp == other.timestamp)
    }
}