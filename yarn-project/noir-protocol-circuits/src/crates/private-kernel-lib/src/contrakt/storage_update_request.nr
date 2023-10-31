use dep::aztec::constants_gen;
struct StorageUpdateRequest{
    storage_slot : Field,
    old_value : Field,
    new_value : Field,
}

impl StorageUpdateRequest {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.storage_slot,
            self.old_value,
            self.new_value,
        ], constants_gen::GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }
    
    fn is_empty(self) -> bool {
        self.storage_slot == 0
    }
}
