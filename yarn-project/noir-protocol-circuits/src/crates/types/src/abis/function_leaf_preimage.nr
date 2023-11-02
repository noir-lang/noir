use crate::abis::function_selector::FunctionSelector;
use dep::aztec::constants_gen;

struct FunctionLeafPreimage {
    selector : FunctionSelector,
    is_internal : bool,
    is_private : bool,
    vk_hash : Field,
    acir_hash : Field
}

impl FunctionLeafPreimage {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.selector.to_field(),
            self.is_internal as Field,
            self.is_private as Field,
            self.vk_hash,
            self.acir_hash
        ], constants_gen::GENERATOR_INDEX__FUNCTION_LEAF)
    }
}
