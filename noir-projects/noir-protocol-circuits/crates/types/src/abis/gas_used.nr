use crate::{
    abis::function_selector::FunctionSelector, address::{EthAddress, AztecAddress},
    constants::GAS_USED_LENGTH, hash::pedersen_hash, traits::{Deserialize, Hash, Serialize, Empty},
    abis::side_effect::Ordered, utils::reader::Reader
};

struct GasUsed {
    da_gas: Field,
    l1_gas: Field,
    l2_gas: Field,
}

impl GasUsed {
    fn new(da_gas: Field, l1_gas: Field, l2_gas: Field) -> Self {
        Self { da_gas, l1_gas, l2_gas }
    }

    fn add(self, other: GasUsed) -> Self {
        GasUsed::new(
            self.da_gas + other.da_gas,
            self.l1_gas + other.l1_gas,
            self.l2_gas + other.l2_gas
        )
    }
}

impl Serialize<GAS_USED_LENGTH> for GasUsed {
    fn serialize(self) -> [Field; GAS_USED_LENGTH] {
        [self.da_gas, self.l1_gas, self.l2_gas]
    }
}

impl Deserialize<GAS_USED_LENGTH> for GasUsed {
    fn deserialize(serialized: [Field; GAS_USED_LENGTH]) -> GasUsed {
        GasUsed::new(serialized[0], serialized[1], serialized[2])
    }
}

impl Eq for GasUsed {
    fn eq(self, other : GasUsed) -> bool {
        (self.da_gas == other.da_gas) & (self.l1_gas == other.l1_gas) & (self.l2_gas == other.l2_gas)
    }
}

impl Empty for GasUsed {
    fn empty() -> Self {
        GasUsed::new(0, 0, 0)
    }
}
