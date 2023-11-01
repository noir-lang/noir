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

}