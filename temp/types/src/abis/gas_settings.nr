use crate::{
    abis::{gas::Gas, gas_fees::GasFees},
    constants::GAS_SETTINGS_LENGTH,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

pub struct GasSettings {
    pub gas_limits: Gas,
    pub teardown_gas_limits: Gas,
    pub max_fees_per_gas: GasFees,
    pub max_priority_fees_per_gas: GasFees,
}

impl GasSettings {
    pub fn new(
        gas_limits: Gas,
        teardown_gas_limits: Gas,
        max_fees_per_gas: GasFees,
        max_priority_fees_per_gas: GasFees,
    ) -> Self {
        Self { gas_limits, teardown_gas_limits, max_fees_per_gas, max_priority_fees_per_gas }
    }
}

impl Eq for GasSettings {
    fn eq(self, other: Self) -> bool {
        (self.gas_limits == other.gas_limits)
            & (self.teardown_gas_limits == other.teardown_gas_limits)
            & (self.max_fees_per_gas == other.max_fees_per_gas)
            & (self.max_priority_fees_per_gas == other.max_priority_fees_per_gas)
    }
}

impl Empty for GasSettings {
    fn empty() -> Self {
        GasSettings::new(
            Gas::empty(),
            Gas::empty(),
            GasFees::empty(),
            GasFees::empty(),
        )
    }
}

impl Serialize<GAS_SETTINGS_LENGTH> for GasSettings {
    fn serialize(self) -> [Field; GAS_SETTINGS_LENGTH] {
        let mut serialized: BoundedVec<Field, GAS_SETTINGS_LENGTH> = BoundedVec::new();

        serialized.extend_from_array(self.gas_limits.serialize());
        serialized.extend_from_array(self.teardown_gas_limits.serialize());
        serialized.extend_from_array(self.max_fees_per_gas.serialize());
        serialized.extend_from_array(self.max_priority_fees_per_gas.serialize());

        serialized.storage()
    }
}

impl Deserialize<GAS_SETTINGS_LENGTH> for GasSettings {
    fn deserialize(serialized: [Field; GAS_SETTINGS_LENGTH]) -> GasSettings {
        let mut reader = Reader::new(serialized);
        GasSettings::new(
            reader.read_struct(Gas::deserialize),
            reader.read_struct(Gas::deserialize),
            reader.read_struct(GasFees::deserialize),
            reader.read_struct(GasFees::deserialize),
        )
    }
}

#[test]
fn serialization_of_empty() {
    let item = GasSettings::empty();
    let serialized = item.serialize();
    let deserialized = GasSettings::deserialize(serialized);
    assert(item.eq(deserialized));
}
