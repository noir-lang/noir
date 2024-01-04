use crate::abis::function_selector::FunctionSelector;
use crate::constants::GENERATOR_INDEX__FUNCTION_DATA;

struct FunctionData {
    // First four bytes of the abi encoding 
    // of a function. 
    selector : FunctionSelector,
    is_internal : bool,
    is_private : bool,
    is_constructor : bool,
}

impl FunctionData {
    // A field is ~256 bits
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3057): Since, function data can fit into a Field,
    // This method will simply return a bit packed Field instead of hashing
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.selector.to_field(),
            self.is_internal as Field,
            self.is_private as Field,
            self.is_constructor as Field,
        ], GENERATOR_INDEX__FUNCTION_DATA)
    }
}
