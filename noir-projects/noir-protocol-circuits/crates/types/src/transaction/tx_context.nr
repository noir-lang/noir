use crate::{
    constants::{GENERATOR_INDEX__TX_CONTEXT, TX_CONTEXT_DATA_LENGTH},
    contrakt::contract_deployment_data::ContractDeploymentData, hash::pedersen_hash,
    traits::{Deserialize, Hash, Serialize}, utils::reader::Reader
};

struct TxContext {
    is_fee_payment_tx : bool,
    is_rebate_payment_tx : bool,
    is_contract_deployment_tx : bool,

    contract_deployment_data : ContractDeploymentData,

    chain_id : Field,
    version : Field,
}

impl Eq for TxContext {
    fn eq(self, other: Self) -> bool {
        (self.is_fee_payment_tx == other.is_fee_payment_tx) &
        (self.is_rebate_payment_tx == other.is_rebate_payment_tx) &
        (self.is_contract_deployment_tx == other.is_contract_deployment_tx) &
        self.contract_deployment_data.eq(other.contract_deployment_data) &
        (self.chain_id == other.chain_id) &
        (self.version == other.version)
    }
}

impl Serialize<TX_CONTEXT_DATA_LENGTH> for TxContext {
    fn serialize(self) -> [Field; TX_CONTEXT_DATA_LENGTH] {
        let mut fields: BoundedVec<Field, TX_CONTEXT_DATA_LENGTH> = BoundedVec::new();

        fields.push(self.is_fee_payment_tx as Field);
        fields.push(self.is_rebate_payment_tx as Field);
        fields.push(self.is_contract_deployment_tx as Field);
        fields.extend_from_array(self.contract_deployment_data.serialize());
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
            is_contract_deployment_tx: reader.read() as bool,
            contract_deployment_data: reader.read_struct(ContractDeploymentData::deserialize),
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
    let context: TxContext = dep::std::unsafe::zeroed();
    let serialized = context.serialize();
    let deserialized = TxContext::deserialize(serialized);
    assert(context.eq(deserialized));
}

#[test]
fn empty_hash() {
    let inputs: TxContext = dep::std::unsafe::zeroed();
    let hash = inputs.hash();

    // Value from contract_deployment_data.test.ts "computes empty item hash" test
    assert_eq(hash, 0x2e7ff14389eef3dc51597529149e01b49cb33829f7089438c8c145c8f352c17b);
}
