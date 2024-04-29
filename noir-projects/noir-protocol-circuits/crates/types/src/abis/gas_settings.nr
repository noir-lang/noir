use crate::{
    abis::function_selector::FunctionSelector, address::{EthAddress, AztecAddress}, abis::gas::Gas,
    abis::gas_fees::GasFees,
    constants::{
    GAS_SETTINGS_LENGTH, DEFAULT_GAS_LIMIT, DEFAULT_TEARDOWN_GAS_LIMIT, DEFAULT_MAX_FEE_PER_GAS,
    DEFAULT_INCLUSION_FEE
},
    hash::pedersen_hash, traits::{Deserialize, Hash, Serialize, Empty}, abis::side_effect::Ordered,
    utils::reader::Reader
};

struct GasSettings {
    gas_limits: Gas,
    teardown_gas_limits: Gas,
    max_fees_per_gas: GasFees,
    inclusion_fee: Field,
}

impl GasSettings {
    pub fn new(
        gas_limits: Gas,
        teardown_gas_limits: Gas,
        max_fees_per_gas: GasFees,
        inclusion_fee: Field
    ) -> Self {
        Self { gas_limits, teardown_gas_limits, max_fees_per_gas, inclusion_fee }
    }

    pub fn default() -> Self {
        GasSettings::new(
            Gas::new(DEFAULT_GAS_LIMIT, DEFAULT_GAS_LIMIT),
            Gas::new(DEFAULT_TEARDOWN_GAS_LIMIT, DEFAULT_TEARDOWN_GAS_LIMIT),
            GasFees::new(DEFAULT_MAX_FEE_PER_GAS, DEFAULT_MAX_FEE_PER_GAS),
            DEFAULT_INCLUSION_FEE
        )
    }
}

impl Eq for GasSettings {
    fn eq(self, other: Self) -> bool {
        (self.gas_limits == other.gas_limits) & (self.teardown_gas_limits == other.teardown_gas_limits) & (self.max_fees_per_gas == other.max_fees_per_gas) & (self.inclusion_fee == other.inclusion_fee)
    }
}

impl Empty for GasSettings {
    fn empty() -> Self {
        GasSettings::new(
            Gas::empty(), Gas::empty(), GasFees::empty(), 0
        )
    }
}

impl Serialize<GAS_SETTINGS_LENGTH> for GasSettings {
    fn serialize(self) -> [Field; GAS_SETTINGS_LENGTH] {
        let mut serialized: BoundedVec<Field, GAS_SETTINGS_LENGTH> = BoundedVec::new();

        serialized.extend_from_array(self.gas_limits.serialize());
        serialized.extend_from_array(self.teardown_gas_limits.serialize());
        serialized.extend_from_array(self.max_fees_per_gas.serialize());
        serialized.push(self.inclusion_fee);
    
        serialized.storage
    }
}

impl Deserialize<GAS_SETTINGS_LENGTH> for GasSettings {
    fn deserialize(serialized: [Field; GAS_SETTINGS_LENGTH]) -> GasSettings {
        let mut reader = Reader::new(serialized);
        GasSettings::new(reader.read_struct(Gas::deserialize), reader.read_struct(Gas::deserialize), reader.read_struct(GasFees::deserialize), reader.read())
    }
}
