use crate::{
    abis::gas_settings::GasSettings,
    constants::TX_CONTEXT_LENGTH,
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

// docs:start:tx-context
pub struct TxContext {
    pub chain_id: Field,
    pub version: Field,
    pub gas_settings: GasSettings,
}
// docs:end:tx-context

impl TxContext {
    pub fn new(chain_id: Field, version: Field, gas_settings: GasSettings) -> Self {
        TxContext { chain_id, version, gas_settings }
    }
}

impl Eq for TxContext {
    fn eq(self, other: Self) -> bool {
        (self.chain_id == other.chain_id)
            & (self.version == other.version)
            & (self.gas_settings.eq(other.gas_settings))
    }
}

impl Empty for TxContext {
    fn empty() -> Self {
        TxContext { chain_id: 0, version: 0, gas_settings: GasSettings::empty() }
    }
}

impl Serialize<TX_CONTEXT_LENGTH> for TxContext {
    fn serialize(self) -> [Field; TX_CONTEXT_LENGTH] {
        let mut fields: BoundedVec<Field, TX_CONTEXT_LENGTH> = BoundedVec::new();

        fields.push(self.chain_id);
        fields.push(self.version);
        fields.extend_from_array(self.gas_settings.serialize());

        assert_eq(fields.len(), TX_CONTEXT_LENGTH);

        fields.storage()
    }
}

impl Deserialize<TX_CONTEXT_LENGTH> for TxContext {
    fn deserialize(serialized: [Field; TX_CONTEXT_LENGTH]) -> Self {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut reader = Reader::new(serialized);

        let context = Self {
            chain_id: reader.read(),
            version: reader.read(),
            gas_settings: reader.read_struct(GasSettings::deserialize),
        };

        reader.finish();
        context
    }
}

#[test]
fn serialization_of_empty() {
    let context = TxContext::empty();
    let serialized = context.serialize();
    let deserialized = TxContext::deserialize(serialized);
    assert(context.eq(deserialized));
}
