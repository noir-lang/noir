use crate::{
    constants::{GENERATOR_INDEX__TX_CONTEXT, TX_CONTEXT_DATA_LENGTH}, hash::pedersen_hash,
    traits::{Deserialize, Hash, Serialize, Empty}, utils::reader::Reader
};

struct TxContext {
    is_fee_payment_tx : bool,
    is_rebate_payment_tx : bool,

    chain_id : Field,
    version : Field,
}

impl Eq for TxContext {
    fn eq(self, other: Self) -> bool {
        (self.is_fee_payment_tx == other.is_fee_payment_tx) &
        (self.is_rebate_payment_tx == other.is_rebate_payment_tx) &
        (self.chain_id == other.chain_id) &
        (self.version == other.version)
    }
}

impl Empty for TxContext {
    fn empty() -> Self {
        TxContext {
            is_fee_payment_tx : false,
            is_rebate_payment_tx : false,
            chain_id : 0,
            version : 0,
        }
    }
}

impl Serialize<TX_CONTEXT_DATA_LENGTH> for TxContext {
    fn serialize(self) -> [Field; TX_CONTEXT_DATA_LENGTH] {
        let mut fields: BoundedVec<Field, TX_CONTEXT_DATA_LENGTH> = BoundedVec::new();

        fields.push(self.is_fee_payment_tx as Field);
        fields.push(self.is_rebate_payment_tx as Field);
        fields.push(self.chain_id);
        fields.push(self.version);

        assert_eq(fields.len(), TX_CONTEXT_DATA_LENGTH);

        fields.storage
    }
}

impl Deserialize<TX_CONTEXT_DATA_LENGTH> for TxContext {
    fn deserialize(serialized: [Field; TX_CONTEXT_DATA_LENGTH]) -> Self {
        // TODO(#4390): This should accept a reader ^ to avoid copying data.
        let mut reader = Reader::new(serialized);

        let context = Self {
            is_fee_payment_tx: reader.read() as bool,
            is_rebate_payment_tx: reader.read() as bool,
            chain_id: reader.read(),
            version: reader.read(),
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
    let inputs = TxContext::empty();
    let hash = inputs.hash();

    // Value from tx_context.test.ts "computes empty item hash" test
    let test_data_empty_hash = 0x200569267c0f73ac89aaa414239398db9445dd4ad3a8cf37015cd55b8d4c5e8d;
    assert_eq(hash, test_data_empty_hash);
}
