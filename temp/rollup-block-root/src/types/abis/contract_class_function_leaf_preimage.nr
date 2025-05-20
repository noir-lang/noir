use crate::types::abis::function_selector::FunctionSelector;
use crate::types::constants::GENERATOR_INDEX__FUNCTION_LEAF;
use crate::types::hash::poseidon2_hash_with_separator;
use crate::types::traits::{Hash, ToField};

pub(crate) struct ContractClassFunctionLeafPreimage {
    pub(crate) selector: FunctionSelector,
    pub(crate) vk_hash: Field,
}

impl Hash for ContractClassFunctionLeafPreimage {
    fn hash(self) -> Field {
        poseidon2_hash_with_separator(
            [self.selector.to_field(), self.vk_hash],
            GENERATOR_INDEX__FUNCTION_LEAF,
        )
    }
}
