use crate::abis::function_selector::FunctionSelector;
use crate::constants::GENERATOR_INDEX__FUNCTION_LEAF;
use crate::traits::Hash;
use crate::hash::poseidon2_hash_with_separator;

struct ContractClassFunctionLeafPreimage {
    selector : FunctionSelector,
    vk_hash : Field,
}

impl Hash for ContractClassFunctionLeafPreimage {
    fn hash(self) -> Field {
        poseidon2_hash_with_separator([
            self.selector.to_field(),
            self.vk_hash,
        ], GENERATOR_INDEX__FUNCTION_LEAF)
    }
}
