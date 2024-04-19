use crate::{
    constants::{GENERATOR_INDEX__TX_CONTEXT, TX_CONTEXT_LENGTH}, hash::pedersen_hash,
    traits::{Deserialize, Hash, Serialize, Empty}, utils::reader::Reader,
    abis::gas_settings::GasSettings
};

// docs:start:tx-context
struct TxContext {
    chain_id : Field,
    version : Field,
    gas_settings: GasSettings,
}
// docs:end:tx-context

impl TxContext {
    pub fn new(chain_id: Field, version: Field, gas_settings: GasSettings) -> Self {
        TxContext { chain_id, version, gas_settings }
    }
}

impl Eq for TxContext {
    fn eq(self, other: Self) -> bool {
        (self.chain_id == other.chain_id) &
        (self.version == other.version) &
        (self.gas_settings.eq(other.gas_settings))
    }
}

impl Empty for TxContext {
    fn empty() -> Self {
        TxContext {
            chain_id: 0,
            version: 0,
            gas_settings: GasSettings::empty(),
        }
    }
}

impl Serialize<TX_CONTEXT_LENGTH> for TxContext {
    fn serialize(self) -> [Field; TX_CONTEXT_LENGTH] {
        let mut fields: BoundedVec<Field, TX_CONTEXT_LENGTH> = BoundedVec::new();

        fields.push(self.chain_id);
        fields.push(self.version);
        fields.extend_from_array(self.gas_settings.serialize());

        assert_eq(fields.len(), TX_CONTEXT_LENGTH);

        fields.storage
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

impl Hash for TxContext {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__TX_CONTEXT)
    }
}

#[test]
fn serialization_of_empty() {
    let context = TxContext::empty();
    let serialized = context.serialize();
    let deserialized = TxContext::deserialize(serialized);
    assert(context.eq(deserialized));
}

#[test]
fn empty_hash() {
    let context = TxContext::empty();
    let hash = context.hash();

    // Value from tx_context.test.ts "computes empty item hash" test
    let test_data_empty_hash = 0x1acd086cc3b911cd49713c263bf4af6032a567fa1e79fe7ef77c063565d3ead3;
    assert_eq(hash, test_data_empty_hash);
}
