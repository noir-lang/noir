use crate::{
    abis::function_selector::FunctionSelector, address::{EthAddress, AztecAddress},
    constants::GAS_LENGTH, hash::pedersen_hash, traits::{Deserialize, Hash, Serialize, Empty},
    abis::side_effect::Ordered, utils::reader::Reader
};

struct Gas {
    da_gas: u32,
    l1_gas: u32,
    l2_gas: u32,
}

impl Gas {
    pub fn new(da_gas: u32, l1_gas: u32, l2_gas: u32) -> Self {
        Self { da_gas, l1_gas, l2_gas }
    }

    fn add(self, other: Gas) -> Self {
        Gas::new(
            self.da_gas + other.da_gas,
            self.l1_gas + other.l1_gas,
            self.l2_gas + other.l2_gas
        )
    }

    fn sub(self, other: Gas) -> Self {
        Gas::new(
            self.da_gas - other.da_gas,
            self.l1_gas - other.l1_gas,
            self.l2_gas - other.l2_gas
        )
    }
}

impl Serialize<GAS_LENGTH> for Gas {
    fn serialize(self) -> [Field; GAS_LENGTH] {
        [self.da_gas as Field, self.l1_gas as Field, self.l2_gas as Field]
    }
}

impl Deserialize<GAS_LENGTH> for Gas {
    fn deserialize(serialized: [Field; GAS_LENGTH]) -> Gas {
        Gas::new(serialized[0] as u32, serialized[1] as u32, serialized[2] as u32)
    }
}

impl Eq for Gas {
    fn eq(self, other : Gas) -> bool {
        (self.da_gas == other.da_gas) & (self.l1_gas == other.l1_gas) & (self.l2_gas == other.l2_gas)
    }
}

impl Empty for Gas {
    fn empty() -> Self {
        Gas::new(0, 0, 0)
    }
}

