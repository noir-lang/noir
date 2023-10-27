use dep::aztec::constants_gen;

struct PublicDataUpdateRequest {
    leaf_index : Field,
    old_value : Field,
    new_value : Field
}

impl PublicDataUpdateRequest {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.leaf_index,
            self.old_value,
            self.new_value
        ], constants_gen::GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }
}
