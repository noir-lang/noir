use crate::{
    abis::function_selector::FunctionSelector, address::{EthAddress, AztecAddress},
    constants::{GAS_SETTINGS_LENGTH, DIMENSION_GAS_SETTINGS_LENGTH}, hash::pedersen_hash,
    traits::{Deserialize, Hash, Serialize, Empty}, abis::side_effect::Ordered, utils::reader::Reader
};

struct GasSettings {
    da: DimensionGasSettings,
    l1: DimensionGasSettings,
    l2: DimensionGasSettings,
    inclusion_fee: Field,
}

impl GasSettings {
    fn new(
        da: DimensionGasSettings,
        l1: DimensionGasSettings,
        l2: DimensionGasSettings,
        inclusion_fee: Field
    ) -> Self {
        Self { da, l1, l2, inclusion_fee }
    }
}

impl Empty for GasSettings {
    fn empty() -> Self {
        GasSettings::new(
            DimensionGasSettings::empty(),
            DimensionGasSettings::empty(),
            DimensionGasSettings::empty(),
            0,
        )
    }
}

impl Serialize<GAS_SETTINGS_LENGTH> for GasSettings {
    fn serialize(self) -> [Field; GAS_SETTINGS_LENGTH] {
        let mut serialized: BoundedVec<Field, GAS_SETTINGS_LENGTH> = BoundedVec::new();

        serialized.extend_from_array(self.da.serialize());
        serialized.extend_from_array(self.l1.serialize());
        serialized.extend_from_array(self.l2.serialize());
        serialized.push(self.inclusion_fee);
    
        serialized.storage
    }
}

impl Deserialize<GAS_SETTINGS_LENGTH> for GasSettings {
    fn deserialize(serialized: [Field; GAS_SETTINGS_LENGTH]) -> GasSettings {
        let mut reader = Reader::new(serialized);
        GasSettings {
            da: reader.read_struct(DimensionGasSettings::deserialize),
            l1: reader.read_struct(DimensionGasSettings::deserialize),
            l2: reader.read_struct(DimensionGasSettings::deserialize),
            inclusion_fee: reader.read(),
        }
    }
}

struct DimensionGasSettings {
    gas_limit: u32,
    teardown_gas_limit: u32,
    max_fee_per_gas: Field,
}

impl DimensionGasSettings {
    fn new(gas_limit: u32, teardown_gas_limit: u32, max_fee_per_gas: Field) -> Self {
        Self { gas_limit, teardown_gas_limit, max_fee_per_gas }
    }
}

impl Serialize<DIMENSION_GAS_SETTINGS_LENGTH> for DimensionGasSettings {
    fn serialize(self) -> [Field; DIMENSION_GAS_SETTINGS_LENGTH] {
        [
            self.gas_limit as Field,
            self.teardown_gas_limit as Field,
            self.max_fee_per_gas,
        ]
    }
}

impl Deserialize<DIMENSION_GAS_SETTINGS_LENGTH> for DimensionGasSettings {
    fn deserialize(serialized: [Field; DIMENSION_GAS_SETTINGS_LENGTH]) -> DimensionGasSettings {
        DimensionGasSettings::new(serialized[0] as u32, serialized[1] as u32, serialized[2])
    }
}

impl Empty for DimensionGasSettings {
    fn empty() -> Self {
        DimensionGasSettings::new(0,0,0)
    }
}
