use crate::{
    abis::gas_fees::GasFees,
    constants::{FIXED_DA_GAS, FIXED_L2_GAS, GAS_LENGTH},
    traits::{Deserialize, Empty, Serialize},
};
use std::ops::{Add, Sub};

pub struct Gas {
    pub da_gas: u32,
    pub l2_gas: u32,
}

impl Gas {
    pub fn new(da_gas: u32, l2_gas: u32) -> Self {
        Self { da_gas, l2_gas }
    }

    pub fn tx_overhead() -> Self {
        Self { da_gas: FIXED_DA_GAS, l2_gas: FIXED_L2_GAS }
    }

    pub fn compute_fee(self, fees: GasFees) -> Field {
        (self.da_gas as Field) * fees.fee_per_da_gas + (self.l2_gas as Field) * fees.fee_per_l2_gas
    }

    pub fn is_empty(self) -> bool {
        (self.da_gas == 0) & (self.l2_gas == 0)
    }

    pub fn within(self, limits: Gas) -> bool {
        (self.da_gas <= limits.da_gas) & (self.l2_gas <= limits.l2_gas)
    }
}

impl Add for Gas {
    fn add(self, other: Gas) -> Self {
        Gas::new(self.da_gas + other.da_gas, self.l2_gas + other.l2_gas)
    }
}

impl Sub for Gas {
    fn sub(self, other: Gas) -> Self {
        Gas::new(self.da_gas - other.da_gas, self.l2_gas - other.l2_gas)
    }
}

impl Serialize<GAS_LENGTH> for Gas {
    fn serialize(self) -> [Field; GAS_LENGTH] {
        [self.da_gas as Field, self.l2_gas as Field]
    }
}

impl Deserialize<GAS_LENGTH> for Gas {
    fn deserialize(serialized: [Field; GAS_LENGTH]) -> Gas {
        Gas::new(serialized[0] as u32, serialized[1] as u32)
    }
}

impl Eq for Gas {
    fn eq(self, other: Gas) -> bool {
        (self.da_gas == other.da_gas) & (self.l2_gas == other.l2_gas)
    }
}

impl Empty for Gas {
    fn empty() -> Self {
        Gas::new(0, 0)
    }
}

#[test]
fn serialization_of_empty() {
    let item = Gas::empty();
    let serialized = item.serialize();
    let deserialized = Gas::deserialize(serialized);
    assert(item.eq(deserialized));
}
