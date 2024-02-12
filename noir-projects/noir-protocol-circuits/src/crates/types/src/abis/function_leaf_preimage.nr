use crate::abis::function_selector::FunctionSelector;
use crate::constants::GENERATOR_INDEX__FUNCTION_LEAF;
use crate::traits::Hash;

struct FunctionLeafPreimage {
    selector : FunctionSelector,
    is_internal : bool,
    is_private : bool,
    vk_hash : Field,
    acir_hash : Field
}

impl Hash for FunctionLeafPreimage {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.selector.to_field(),
            self.is_internal as Field,
            self.is_private as Field,
            self.vk_hash,
            self.acir_hash
        ], GENERATOR_INDEX__FUNCTION_LEAF)
    }
}

struct ContractClassFunctionLeafPreimage {
    selector : FunctionSelector,
    vk_hash : Field,
}

impl Hash for ContractClassFunctionLeafPreimage {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.selector.to_field(),
            self.vk_hash,
        ], GENERATOR_INDEX__FUNCTION_LEAF)
    }
}
